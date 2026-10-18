import { createHash } from "node:crypto";
import { PublicKey } from "@solana/web3.js";

import { u64 } from "./programs.js";

const sha256 = (...parts: Buffer[]) =>
  createHash("sha256").update(Buffer.concat(parts)).digest();

/** Matches `sale::merkle::leaf`: sha256(0x00 || wallet || max_amount as u64 LE). */
export const leaf = (wallet: PublicKey, maxAmount: bigint) =>
  sha256(Buffer.from([0]), wallet.toBuffer(), u64(maxAmount));

/** Matches `sale::merkle::hash_pair`: sha256(0x01 || min || max). */
export const hashPair = (a: Buffer, b: Buffer) =>
  Buffer.compare(a, b) <= 0
    ? sha256(Buffer.from([1]), a, b)
    : sha256(Buffer.from([1]), b, a);

export const verify = (proof: Buffer[], root: Buffer, node: Buffer) =>
  proof.reduce((acc, sibling) => hashPair(acc, sibling), node).equals(root);

export interface AllowlistEntry {
  wallet: PublicKey;
  maxAmount: bigint;
}

/** Root and per-wallet proofs. An odd node is carried up unpaired, as in `sale::merkle::build`. */
export function allowlist(entries: AllowlistEntry[]) {
  if (entries.length === 0) throw new Error("allowlist is empty");
  const proofs: Buffer[][] = entries.map(() => []);
  const positions = entries.map((_, i) => i);
  let level: Buffer[] = entries.map((e) => leaf(e.wallet, e.maxAmount));
  while (level.length > 1) {
    positions.forEach((position, i) => {
      const sibling = position ^ 1;
      if (sibling < level.length) proofs[i].push(level[sibling]);
      positions[i] = position >> 1;
    });
    const next: Buffer[] = [];
    for (let i = 0; i < level.length; i += 2) {
      next.push(i + 1 < level.length ? hashPair(level[i], level[i + 1]) : level[i]);
    }
    level = next;
  }
  const root = level[0];
  return {
    root,
    proofFor(wallet: PublicKey) {
      const index = entries.findIndex((e) => e.wallet.equals(wallet));
      if (index < 0) throw new Error(`${wallet.toBase58()} is not on the allowlist`);
      return { maxAmount: entries[index].maxAmount, proof: proofs[index] };
    },
  };
}
