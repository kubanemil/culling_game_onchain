import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { findPDA, getRent } from "./helpers";
import { Game } from "../target/types/game";
import { assert, expect } from "chai";

describe("game", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.Game as Program<Game>;
  const conn = new web3.Connection(provider.connection.rpcEndpoint, {
    commitment: "confirmed",
  });

  anchor.setProvider(provider);

  // test variables
  const user = provider.publicKey;
  const auth = Keypair.generate();
  const opponent = Keypair.generate();
  const gameId = 923764;
  const stake = new BN(10 ** 9);

  const gameIdBuffer = Buffer.alloc(4);
  gameIdBuffer.writeUInt32LE(gameId, 0);
  const [gameAddress] = findPDA(
    [Buffer.from("game"), gameIdBuffer, user.toBuffer()],
    program.programId
  );
  const [configAddress] = findPDA([Buffer.from("config")], program.programId);
  const [creatorPlayerAddress] = findPDA(
    [Buffer.from("player"), user.toBuffer()],
    program.programId
  );
  const [opponentPlayerAddress] = findPDA(
    [Buffer.from("player"), opponent.publicKey.toBuffer()],
    program.programId
  );

  it("initialize config", async () => {
    // provide auth wallet with SOL to create it
    const airdrop_tx = await provider.connection.requestAirdrop(
      auth.publicKey,
      1 * LAMPORTS_PER_SOL
    );
    await conn.confirmTransaction(airdrop_tx);

    const tx = await program.methods
      .initConfig()
      .accounts({
        auth: auth.publicKey,
      })
      .rpc();
    conn.confirmTransaction(tx, "confirmed");

    // confirm game pda info
    const game = await program.account.config.fetch(configAddress);
    assert(game.auth.equals(auth.publicKey), "invalid config auth");
    assert(game.owner.equals(user), "invalid config owner");
  });

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
    assert(!game.accepted, "game should not be accepted");
    assert(game.creator.equals(user), "invalid game creator");
    assert(game.opponent.equals(opponent.publicKey), "invalid game opponent");
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
    expect(userInitBalance - userBalance).closeTo(
      stake.toNumber(),
      2 * 10 ** 6
    );

    // check game status changed
    const game = await program.account.game.fetch(gameAddress);
    assert(game.accepted, "Game is not accepted");
  });

  it("resolve game by non-auth signer", async () => {
    try {
      await program.methods
        .resolveGame(gameId)
        .accounts({
          auth: opponent.publicKey,
          creator: user,
          winner: opponent.publicKey,
          game: gameAddress,
        })
        .signers([opponent])
        .rpc();

      expect.fail("Instruction called by non-auth signer should fail");
    } catch (e: any) {
      expect(e.message).contains(
        "Error Code: NotAuthority. Error Number: 6003."
      );
    }
  });

  it("resolve game", async () => {
    const winner = opponent.publicKey;

    const winnerInitBalance = await conn.getBalance(winner, "processed");
    const gameInitBalance = await conn.getBalance(gameAddress, "processed");
    expect(gameInitBalance).gte(stake.toNumber() * 2);

    const creatorPlayer = await program.account.player.fetch(
      creatorPlayerAddress
    );
    expect(creatorPlayer.initiated).eq(true);
    expect(creatorPlayer.owner.equals(user)).eq(true);
    expect(creatorPlayer.gameWon + creatorPlayer.gameLost).eq(0);

    const opponentPlayer = await program.account.player.fetch(
      opponentPlayerAddress
    );
    expect(opponentPlayer.initiated).eq(true);
    expect(opponentPlayer.owner.equals(opponent.publicKey)).eq(true);
    expect(opponentPlayer.gameWon + creatorPlayer.gameLost).eq(0);

    // invoke ix
    await program.methods
      .resolveGame(gameId)
      .accounts({
        auth: auth.publicKey,
        creator: user,
        winner: winner,
        game: gameAddress,
      })
      .signers([auth])
      .rpc();

    // check if game transfered a stake to winner
    const gameBalance = await conn.getBalance(gameAddress, "processed");
    const winnerBalance = await conn.getBalance(winner, "processed");
    expect(gameBalance).eq(0, "Game balance is not drained");
    expect(winnerBalance - winnerInitBalance).gte(
      2 * stake.toNumber(),
      "winner received invalid stake"
    );

    // check player accounts
    const creatorPlayerAfter = await program.account.player.fetch(
      creatorPlayerAddress
    );
    expect(creatorPlayerAfter.initiated).eq(true);
    expect(creatorPlayerAfter.owner.equals(user)).eq(true);
    expect(creatorPlayerAfter.gameWon).eq(0);
    expect(creatorPlayerAfter.gameLost).eq(1);

    const opponentPlayerAfter = await program.account.player.fetch(
      opponentPlayerAddress
    );
    expect(opponentPlayerAfter.initiated).eq(true);
    expect(opponentPlayerAfter.owner.equals(opponent.publicKey)).eq(true);
    expect(opponentPlayerAfter.gameWon).eq(1);
    expect(opponentPlayerAfter.gameLost).eq(0);
  });
});
