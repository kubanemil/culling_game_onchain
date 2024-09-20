import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { findPDA, getLogs } from "./helpers";
import { Game } from "../target/types/game";

const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

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
  const stakeAmount = new BN(10 ** 7);

  const gameIdBuffer = Buffer.alloc(4);
  gameIdBuffer.writeUInt32LE(gameId, 0);

  const [gameAddress] = findPDA(
    [Buffer.from("game"), gameIdBuffer, user.toBuffer()],
    program.programId
  );
  await provider.connection.requestAirdrop(opponent.publicKey, 10*LAMPORTS_PER_SOL)

  it("create game", async () => {
    // Add your test here.
    const tx = await program.methods
      .createGame(gameId, stakeAmount)
      .accounts({
        opponent: opponent.publicKey,
        game: gameAddress,
      })
      .rpc();
  });

  it("accept game", async () => {
    // Add your test here.
    const tx = await program.methods
      .acceptGame(gameId)
      .accounts({
        signer: opponent.publicKey,
        initiator: user,
        game: gameAddress,
      })
      .signers([opponent])
      .rpc();
  });
});
