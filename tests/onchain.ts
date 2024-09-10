import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Asset } from "../target/types/asset";
import { expect, assert } from "chai";


describe("onchain", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Asset as Program<Asset>;

  const vault = anchor.web3.Keypair.generate();

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().accounts(
      {
        "tokenProgram": TOKEN_PROGRAM_ID
      }
    ).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Buy tokens", async () => {
    // await program.methods.initialize().accounts({"tokenProgram": TOKEN_PROGRAM_ID}).rpc();
    const amount = new anchor.BN(1000000);
    const tx = await program.methods.buyToken(amount).rpc();
  } )
});
