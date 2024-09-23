import { Keypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { findPDA } from "./helpers";
import { Governance } from "../target/types/governance";
import { assert, expect } from "chai";

describe("governance", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
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

  it("initialize config", async () => {
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
});
