import { createHash, randomBytes } from "node:crypto";
import { PublicKey } from "@solana/web3.js";

/**
 * 32 bytes from a CSPRNG. Cards are few enough to guess, so the secret is the only thing
 * hiding your card from the opponent until you reveal.
 */
export const newSecret = () => randomBytes(32);

/** Matches `duel::commitment`: sha256(duel || player || card mint || secret). */
export const commitment = (
  duel: PublicKey,
  player: PublicKey,
  cardMint: PublicKey,
  secret: Buffer,
) =>
  createHash("sha256")
    .update(Buffer.concat([duel.toBuffer(), player.toBuffer(), cardMint.toBuffer(), secret]))
    .digest();
