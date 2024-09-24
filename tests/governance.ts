import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { findPDA, getATA, getVaultMintAddress } from "./helpers";
import { Governance } from "../target/types/governance";
import { assert, expect } from "chai";
import { Asset } from "../target/types/asset";
import { getAccount, getMint } from "@solana/spl-token";
import { fromWeb3JsPublicKey } from "@metaplex-foundation/umi-web3js-adapters";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import {
  createSignerFromKeypair,
  keypairIdentity,
} from "@metaplex-foundation/umi";
import {
  fetchDigitalAsset,
  findMetadataPda,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";

async function skipSlots(conn: Connection, slots: number) {
  for (let i = 0; i < slots; i++) {
    let payer = Keypair.generate();
    const airdrop_tx = await conn.requestAirdrop(
      payer.publicKey,
      LAMPORTS_PER_SOL
    );
    await conn.confirmTransaction(airdrop_tx, "confirmed");
    console.log(`slot #${await conn.getSlot("confirmed")}`);
  }
}

describe("governance", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const asset_program = anchor.workspace.Asset as Program<Asset>;
  const program = anchor.workspace.Governance as Program<Governance>;
  const conn = new web3.Connection(provider.connection.rpcEndpoint, {
    commitment: "confirmed",
  });

  const umi = createUmi(conn);
  const payer = provider.wallet as NodeWallet;
  const creatorWallet = umi.eddsa.createKeypairFromSecretKey(
    new Uint8Array(payer.payer.secretKey)
  );
  const creator = createSignerFromKeypair(umi, creatorWallet);

  umi.use(keypairIdentity(creator));
  umi.use(mplTokenMetadata());
  anchor.setProvider(provider);

  // test variables
  const user = provider.publicKey;
  const amendmentId = 7;
  const cardId = 5;
  const newMetadataUri = "https://chat.lab.epam.com/";
  const deadlineSlot = new BN(10);

  const [amendment] = findPDA(
    [Buffer.from("amendment"), user.toBuffer()],
    program.programId
  );

  const [voteAddress] = await findPDA(
    [Buffer.from("vote"), amendment.toBuffer()],
    program.programId
  );
  const [vault, mint_address] = await getVaultMintAddress(asset_program);
  const [cardAddress] = findPDA(
    [
      Buffer.from("card"),
      Buffer.from(new Uint8Array([cardId])),
      user.toBuffer(),
    ],
    asset_program.programId
  );
  const card = fromWeb3JsPublicKey(cardAddress);
  const [cardMetadataAddress] = findMetadataPda(umi, {
    mint: card,
  });

  it("init assets and cards", async () => {
    // init assets
    const init_tx = await asset_program.methods.initialize().rpc();
    await conn.confirmTransaction(init_tx, "confirmed");

    // create card
    const init_card_tx = await asset_program.methods
      .initCard(cardId)
      .accounts({
        card: cardAddress,
      })
      .rpc();
    await conn.confirmTransaction(init_card_tx, "confirmed");

    // create card metadata
    const card_meta_tx = await asset_program.methods
      .createMetadata(cardId, "cullingMeta", "CT@", "https://excalidraw.com/")
      .accounts({
        signer: user,
        card: cardAddress,
        metadata: cardMetadataAddress,
      })
      .rpc();
    await conn.confirmTransaction(card_meta_tx, "confirmed");

    // purchase tokens
    const amount = 10_000_000; // is 10 tokens
    const tx = await asset_program.methods.buyToken(new BN(amount)).rpc();
    await conn.confirmTransaction(tx, "confirmed");
  });

  it("create amendment", async () => {
    await program.methods
      .createAmendment(cardId, newMetadataUri, deadlineSlot)
      .accounts({})
      .rpc();

    const amend = await program.account.amendment.fetch(amendment);
    expect(amend.creator.equals(user)).eq(true, "invalid amendment ID");
    expect(amend.cardId).eq(cardId, "invalid cardId");
    expect(amend.newMetadataUri).eq(newMetadataUri, "invalid metadata");
    assert(amend.deadlineSlot.eq(deadlineSlot), "invalid deadline");
  });

  it("vote for amendment", async () => {
    const tokenAmount = new BN(10 ** 6);

    const tx = await program.methods
      .vote(true, tokenAmount)
      .accounts({
        mint: mint_address,
        amendment: amendment,
      })
      .rpc();
    await conn.confirmTransaction(tx, "confirmed");

    // check if vote PDA is created properly
    const vote = await program.account.vote.fetch(voteAddress);
    expect(vote.voter.equals(user)).eq(true, "invalid voter");
    expect(vote.amendment.equals(amendment)).eq(true, "invalid amendment");
    expect(vote.accept).eq(true, "vote should be accepted");
    expect(vote.tokens.eq(tokenAmount)).eq(true, "invalid token amount");

    // check if funds for transfered to vote PDA
    const voteATA = getATA(mint_address, voteAddress, true);
    const voteATAInfo = await getAccount(conn, voteATA);
    expect(voteATAInfo.amount.toString()).eq(
      tokenAmount.toString(),
      "Transfer amount is invalid"
    );

    // check if amendment voting is updated
    const amend = await program.account.amendment.fetch(amendment);
    expect(amend.pros.eq(tokenAmount)).eq(true, "invalid pros");
    expect(amend.cons.eq(new BN(0))).eq(true, "invalid cons");
  });

  it("resolve amendment", async () => {
    await skipSlots(conn, 3);
    const tx = await program.methods
      .resolveAmendment()
      .accounts({
        card: cardAddress,
        vault: vault,
        metadata: cardMetadataAddress,
      })
      .rpc();
    await conn.confirmTransaction(tx, "confirmed");

    // check if metadata uri is updated
    const cardMetadata = await fetchDigitalAsset(umi, card);
    expect(cardMetadata.metadata.uri).eq(newMetadataUri, "Invalid URI");
  });
});
