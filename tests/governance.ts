import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { findPDA, getATA, getVaultMintAddress } from "./helpers";
import { Governance } from "../target/types/governance";
import { assert, expect } from "chai";
import { Asset } from "../target/types/asset";
import { getAccount, getMint } from "@solana/spl-token";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";

describe("governance", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const asset_program = anchor.workspace.Asset as Program<Asset>;
  const program = anchor.workspace.Governance as Program<Governance>;
  const conn = new web3.Connection(provider.connection.rpcEndpoint, {
    commitment: "confirmed",
  });

  anchor.setProvider(provider);

  // test variables
  const user = provider.publicKey;
  const amendmentId = 7;

  const [amendmentAddress] = findPDA(
    [Buffer.from("amendment"), user.toBuffer()],
    program.programId
  );

  const amendmentIdBuffer = Buffer.alloc(4);
  amendmentIdBuffer.writeUInt32LE(amendmentId, 0);
  const [voteAddress] = await findPDA(
    [Buffer.from("vote"), user.toBuffer(), amendmentIdBuffer],
    program.programId
  );
  const [vault, mint_address] = await getVaultMintAddress(asset_program);

  it("create amendment", async () => {
    const cardId = 5;
    const newMetadata = Keypair.generate();
    const deadlineSlot = new BN(10);

    await program.methods
      .createAmendment(amendmentId, cardId, newMetadata.publicKey, deadlineSlot)
      .rpc();

    const amend = await program.account.amendment.fetch(amendmentAddress);
    expect(amend.id).eq(amendmentId, "invalid amendment ID");
    expect(amend.cardId).eq(cardId, "invalid cardId");
    assert(amend.newMetadata.equals(newMetadata.publicKey), "invalid metadata");
    assert(amend.deadlineSlot.eq(deadlineSlot), "invalid deadline");
  });

  it("purchase culling tokens", async () => {
    // init assets
    const init_tx = await asset_program.methods.initialize().rpc();
    await conn.confirmTransaction(init_tx, "confirmed");

    // purchase tokens
    const amount = 10_000_000; // is 10 tokens
    const tx = await asset_program.methods.buyToken(new BN(amount)).rpc();
    await conn.confirmTransaction(tx, "confirmed");
  });

  it("vote for amendment", async () => {
    const tokenAmount = new BN(10 ** 6);

    const tx = await program.methods
      .vote(amendmentId, true, tokenAmount)
      .accounts({
        vote: voteAddress,
        mint: mint_address,
      })
      .rpc();
    await conn.confirmTransaction(tx, "confirmed");

    // check if vote PDA is created properly
    const vote = await program.account.vote.fetch(voteAddress);
    expect(vote.voter.equals(user)).eq(true, "invalid voter");
    expect(vote.amendmentId).eq(amendmentId, "invalid amendment");
    expect(vote.accept).eq(true, "vote should be accepted");
    expect(vote.tokens.eq(tokenAmount)).eq(true, "invalid token amount");

    // check if funds for transfered to vote PDA
    const voteATA = getATA(mint_address, voteAddress, true);
    const voteATAInfo = await getAccount(conn, voteATA);
    expect(voteATAInfo.amount.toString()).eq(
      tokenAmount.toString(),
      "Transfer amount is invalid"
    );
  });
});
