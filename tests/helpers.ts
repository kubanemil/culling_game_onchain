import { web3, Program, Idl } from "@coral-xyz/anchor";
import { Asset } from "../target/types/asset";
import { Connection } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

const vaultSeedStr = "authVault";
const mintSeedStr = "cullingToken";

export const findPDA = (seeds: Array<Buffer>, programId: web3.PublicKey) =>
  web3.PublicKey.findProgramAddressSync(seeds, programId);

export const getATA = (
  mint: web3.PublicKey,
  owner: web3.PublicKey,
  allowOwnerOffCurve?: boolean
) => getAssociatedTokenAddressSync(mint, owner, allowOwnerOffCurve);

export const getRent = (conn: Connection, size: number) =>
  conn.getMinimumBalanceForRentExemption(size);

export const getVaultMintAddress = async (
  program: Program<Asset>
): Promise<[web3.PublicKey, web3.PublicKey]> => {
  const vault_seeds = [Buffer.from(vaultSeedStr)];
  const [vault_address] = findPDA(vault_seeds, program.programId);

  // to get mint you need vault address
  const mint_seeds = [Buffer.from(mintSeedStr), vault_address.toBuffer()];
  const [mint_address] = findPDA(mint_seeds, program.programId);

  return [vault_address, mint_address];
};

export const getLogs = async (
  program: Program<Asset>,
  tx: string
): Promise<Array<string> | null> => {
  const txDetails = await program.provider.connection.getTransaction(tx, {
    maxSupportedTransactionVersion: 0,
    commitment: "confirmed",
  });
  return txDetails?.meta?.logMessages || null;
};
