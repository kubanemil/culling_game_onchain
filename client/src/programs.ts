import { PublicKey } from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";

import assetIdl from "./idl/asset.json" with { type: "json" };
import duelIdl from "./idl/duel.json" with { type: "json" };
import governanceIdl from "./idl/governance.json" with { type: "json" };
import saleIdl from "./idl/sale.json" with { type: "json" };

export { assetIdl, duelIdl, governanceIdl, saleIdl };

export const SALE_PROGRAM_ID = new PublicKey(saleIdl.address);
export const ASSET_PROGRAM_ID = new PublicKey(assetIdl.address);
export const DUEL_PROGRAM_ID = new PublicKey(duelIdl.address);
export const GOVERNANCE_PROGRAM_ID = new PublicKey(governanceIdl.address);

const seed = (s: string) => Buffer.from(s);

export const u64 = (n: bigint | number) => {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(BigInt(n));
  return b;
};

const u16 = (n: number) => {
  const b = Buffer.alloc(2);
  b.writeUInt16LE(n);
  return b;
};

const find = (seeds: Buffer[], program: PublicKey) =>
  PublicKey.findProgramAddressSync(seeds, program)[0];

/** Every PDA in the system, derived the same way the programs derive them. */
export const pda = {
  saleConfig: () => find([seed("config")], SALE_PROGRAM_ID),
  tokenMint: () => find([seed("mint")], SALE_PROGRAM_ID),
  round: (id: bigint | number) => find([seed("round"), u64(id)], SALE_PROGRAM_ID),
  allocation: (round: PublicKey, buyer: PublicKey) =>
    find([seed("allocation"), round.toBuffer(), buyer.toBuffer()], SALE_PROGRAM_ID),

  assetConfig: () => find([seed("config")], ASSET_PROGRAM_ID),
  cardMint: (id: number) => find([seed("card_mint"), u16(id)], ASSET_PROGRAM_ID),
  card: (mint: PublicKey) => find([seed("card"), mint.toBuffer()], ASSET_PROGRAM_ID),

  duel: (creator: PublicKey, nonce: bigint | number) =>
    find([seed("duel"), creator.toBuffer(), u64(nonce)], DUEL_PROGRAM_ID),
  escrow: (duel: PublicKey, player: PublicKey) =>
    find([seed("escrow"), duel.toBuffer(), player.toBuffer()], DUEL_PROGRAM_ID),

  governanceConfig: () => find([seed("config")], GOVERNANCE_PROGRAM_ID),
  governanceAuthority: () => find([seed("authority")], GOVERNANCE_PROGRAM_ID),
  proposal: (id: bigint | number) =>
    find([seed("proposal"), u64(id)], GOVERNANCE_PROGRAM_ID),
  voter: (owner: PublicKey) => find([seed("voter"), owner.toBuffer()], GOVERNANCE_PROGRAM_ID),
  voteRecord: (proposal: PublicKey, owner: PublicKey) =>
    find([seed("vote"), proposal.toBuffer(), owner.toBuffer()], GOVERNANCE_PROGRAM_ID),
  /** `field` is the CardField index: 0 name, 1 symbol, 2 uri. */
  fieldVersion: (cardMint: PublicKey, field: number) =>
    find([seed("field"), cardMint.toBuffer(), Buffer.from([field])], GOVERNANCE_PROGRAM_ID),

  programData: (program: PublicKey) =>
    find(
      [program.toBuffer()],
      new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111"),
    ),
};

/** All mints are Token-2022, so every token account is a Token-2022 ATA. */
export const ata = (owner: PublicKey, mint: PublicKey) =>
  getAssociatedTokenAddressSync(mint, owner, true, TOKEN_2022_PROGRAM_ID);
