import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import {
  getAssociatedTokenAddressSync,
  getAccount,
  getMint,
} from "@solana/spl-token";
import { findPDA, getLogs, getVaultMintAddress } from "./helpers";
import { Asset } from "../target/types/asset";
import { expect, assert } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";

describe("asset", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.Asset as Program<Asset>;
  const user = provider.publicKey;
  const conn = new web3.Connection(provider.connection.rpcEndpoint, {
    commitment: "confirmed",
  });

  anchor.setProvider(provider);

  it("init", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc({
      commitment: "confirmed",
    });

    const logs = await getLogs(program, tx);
    console.log(logs[10]);
  });

  it("Buy tokens", async () => {
    const amount = 20_000_000; // is 1 token, as token::decimals = 6
    const [vault, mint_address] = await getVaultMintAddress(program, user);

    const vaultBalanceBefore = await conn.getBalance(vault);
    // invoke instruction
    await program.methods.buyToken(new BN(amount)).rpc({
      commitment: "confirmed",
    });

    // check balance
    const vaultBalanceAfter = await conn.getBalance(vault);
    expect(vaultBalanceAfter - vaultBalanceBefore).equals(1000 * amount);

    // check mint
    const mintAccount = await getMint(conn, mint_address);
    assert(mintAccount.decimals == 6, "Mint has wrong decimals");
    assert(mintAccount.mintAuthority.equals(vault), "Wrong mint authority");
    assert(mintAccount.supply == BigInt(amount), "Mint supply is invalid");

    // check token
    const ata_address = await getAssociatedTokenAddressSync(mint_address, user);
    const ata = await getAccount(conn, ata_address);
    assert(ata.mint.equals(mint_address), "Wrong ATA mint");
    assert(ata.owner.equals(user), "ATA's owner is invalid.");
    assert(ata.amount == BigInt(amount), "ATA's balance is invalid");
  });

  it("buy a card", async () => {
    const cardId = 7;

    const cardBuffer = Buffer.alloc(1);
    cardBuffer.writeUInt8(cardId);
    const [cardAddress] = findPDA(
      [Buffer.from("card"), cardBuffer, user.toBuffer()],
      program.programId
    );
    console.log("card address: ", cardAddress.toString());

    const tx = await program.methods
      .buyCard(cardId)
      .accounts({
        card: cardAddress,
      })
      .rpc();
    const logs = await getLogs(program, tx);
    console.log(logs);
  });
});
