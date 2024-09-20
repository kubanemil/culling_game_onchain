import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { findPDA, getRent } from "./helpers";
import { Game } from "../target/types/game";
import { assert, expect } from "chai";

describe("game", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.Game as Program<Game>;
  const user = provider.publicKey;
  const conn = new web3.Connection(provider.connection.rpcEndpoint, {
    commitment: "confirmed",
  });

  anchor.setProvider(provider);

  const opponent = Keypair.generate();
  const gameId = 923764;
  const stake = new BN(10 ** 9);

  const gameIdBuffer = Buffer.alloc(4);
  gameIdBuffer.writeUInt32LE(gameId, 0);

  const [gameAddress] = findPDA(
    [Buffer.from("game"), gameIdBuffer, user.toBuffer()],
    program.programId
  );

  it("create game", async () => {
    // aidrop some lamports to opponent
    const airdrop_tx = await provider.connection.requestAirdrop(
      opponent.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await conn.confirmTransaction(airdrop_tx);

    const gameInitBalance = await conn.getBalance(gameAddress, "processed");

    const tx = await program.methods
      .createGame(gameId, stake)
      .accounts({
        opponent: opponent.publicKey,
        game: gameAddress,
      })
      .rpc();
    conn.confirmTransaction(tx, "confirmed");

    // check if game received a stake
    const gameRent = await getRent(conn, program.account.game.size);
    const gameBalance = await conn.getBalance(gameAddress, "processed");
    expect(gameBalance - gameRent - gameInitBalance).eq(
      stake.toNumber(),
      "Invalid game funds"
    );

    // confirm game pda info
    const game = await program.account.game.fetch(gameAddress);
    expect(game.id).eq(gameId, "invalid gameId");
    expect(game.stakeAmount.toNumber()).eq(
      stake.toNumber(),
      "invalid game stake"
    );
    expect(game.accepted).eq(false, "game should not be accepted");
    expect(game.players[0].equals(user)).eq(true, "invalid game creator");
    expect(game.players[1].equals(opponent.publicKey)).eq(
      true,
      "invalid game opponent"
    );
  });

  it("accept game", async () => {
    const userInitBalance = await conn.getBalance(
      opponent.publicKey,
      "processed"
    );

    await program.methods
      .acceptGame(gameId)
      .accounts({
        signer: opponent.publicKey,
        initiator: user,
        game: gameAddress,
      })
      .signers([opponent])
      .rpc();

    // check if user transfered funds
    const userBalance = await conn.getBalance(opponent.publicKey, "processed");
    expect(userInitBalance - userBalance).closeTo(stake.toNumber(), 10 ** 6);

    // check game status changed
    const game = await program.account.game.fetch(gameAddress);
    assert(game.accepted, "Game is not accepted");
  });

  it("resolve game", async () => {
    const gameInitBalance = await conn.getBalance(gameAddress, "processed");
    expect(gameInitBalance).gte(stake.toNumber() * 2);

    const tx = await program.methods
      .resolveGame(gameId)
      .accounts({
        game: gameAddress,
      })
      .rpc();

    // check if game received a stake
    const gameBalance = await conn.getBalance(gameAddress, "processed");
    expect(gameBalance).eq(0, "Game balance is not drained");
  });
});
