// Needs the four programs deployed with the provider wallet as their upgrade authority.
import { AnchorProvider } from "@anchor-lang/core";
import { getTokenMetadata, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  Transaction,
} from "@solana/web3.js";

import { allowlist, Culling, pda } from "../src/index.js";

const TOKEN = 1_000_000n;
const env = AnchorProvider.env();
const provider = new AnchorProvider(
  new Connection(env.connection.rpcEndpoint, "confirmed"),
  env.wallet,
  { commitment: "confirmed" },
);
const culling = new Culling(provider);
const { connection } = provider;

const step = (message: string) => console.log(`\n== ${message}`);
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const exists = async (address: PublicKey) => (await connection.getAccountInfo(address)) !== null;

/** The cluster's clock, which is what the programs compare against. */
async function chainTime() {
  const clock = await connection.getAccountInfo(SYSVAR_CLOCK_PUBKEY);
  return Number(clock!.data.readBigInt64LE(32));
}

async function waitUntil(timestamp: number) {
  while ((await chainTime()) < timestamp) await sleep(2_000);
}

async function fund(to: PublicKey, sol: number) {
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: culling.admin,
      toPubkey: to,
      lamports: sol * LAMPORTS_PER_SOL,
    }),
  );
  await provider.sendAndConfirm(tx);
}

async function setUp() {
  step("Initialize the programs (skipped if already done)");
  if (!(await exists(pda.saleConfig()))) {
    await culling.initializeSale({
      decimals: 6,
      supplyCap: 1_000_000n * TOKEN,
      name: "Culling Token",
      symbol: "CULL",
      uri: "https://example.com/cull.json",
    });
  }
  if (!(await exists(pda.assetConfig()))) await culling.initializeAsset();
  if (!(await exists(pda.governanceConfig()))) {
    await culling.initializeGovernance({ quorum: 10n * TOKEN, thresholdBps: 5_000, votingPeriod: 60 });
  }
  const { cardCount } = await culling.asset.account.config.fetch(pda.assetConfig());
  if (cardCount === 0) {
    for (const [name, stats] of [
      ["Brute", { attack: 20, defense: 2, health: 34 }],
      ["Guard", { attack: 7, defense: 11, health: 54 }],
    ] as const) {
      await culling.createCard({
        stats,
        price: 10n * TOKEN,
        name,
        symbol: "CARD",
        uri: `https://example.com/cards/${name.toLowerCase()}.json`,
      });
    }
  }
  console.log("token mint", pda.tokenMint().toBase58());
  return { brute: pda.cardMint(0), guard: pda.cardMint(1) };
}

async function main() {
  const { brute, guard } = await setUp();

  step("Two new players, funded by the provider wallet");
  const alice = Keypair.generate();
  const bob = Keypair.generate();
  await fund(alice.publicKey, 0.2);
  await fund(bob.publicKey, 0.2);
  console.log("alice", alice.publicKey.toBase58());
  console.log("bob  ", bob.publicKey.toBase58());

  step("Sale: allowlisted round, buy, claim after it ends");
  const list = allowlist([
    { wallet: alice.publicKey, maxAmount: 50n * TOKEN },
    { wallet: bob.publicKey, maxAmount: 50n * TOKEN },
  ]);
  const start = await chainTime();
  const round = await culling.createRound({
    startTs: start,
    endTs: start + 20,
    price: 100_000n,
    hardCap: 100n * TOKEN,
    merkleRoot: list.root,
    cliff: 0,
    vesting: 0,
  });
  for (const player of [alice, bob]) {
    const { maxAmount, proof } = list.proofFor(player.publicKey);
    await culling.buy(player, round, maxAmount, maxAmount, proof);
  }
  console.log(`round ${round}: both bought 50 CULL, waiting for it to end`);
  await waitUntil(start + 20);
  await culling.claim(alice, round);
  await culling.claim(bob, round);
  await culling.withdrawProceeds(round);
  await culling.finalizeRound(round);
  console.log("claimed; admin withdrew the proceeds; round finalized");

  step("Cards: Alice buys Brute, Bob buys Guard (10 CULL each, burned)");
  await culling.buyCard(alice, brute);
  await culling.buyCard(bob, guard);

  step("Duel: commit, accept, reveal (cards go into escrow), claim");
  const stake = 10_000_000n;
  const { duel, secret: aliceSecret } = await culling.createDuel(alice, brute, stake, bob.publicKey);
  const bobSecret = await culling.acceptDuel(bob, duel, guard);
  await culling.reveal(alice, duel, brute, aliceSecret);
  await culling.reveal(bob, duel, guard, bobSecret);
  const escrowRent = await connection.getBalance(pda.escrow(duel, bob.publicKey));
  const before = await connection.getBalance(bob.publicKey);
  await culling.claimDuel(bob, duel);
  await culling.claimDuel(alice, duel);
  const won = (await connection.getBalance(bob.publicKey)) - before - escrowRent;
  const outcome = won === Number(2n * stake) ? "Bob wins" : won === Number(stake) ? "draw" : "Alice wins";
  console.log(`duel ${duel.toBase58()}: ${outcome}; both cards are back`);

  step("Governance: deposit, propose a new URI for Brute, vote, execute");
  await culling.deposit(alice, 40n * TOKEN);
  await culling.deposit(bob, 40n * TOKEN);
  const newUri = "https://example.com/cards/brute-v2.json";
  const id = await culling.createProposal(alice, brute, "uri", newUri);
  await culling.vote(alice, id, true);
  await culling.vote(bob, id, true);
  const { endTs } = await culling.governance.account.proposal.fetch(pda.proposal(id));
  console.log(`proposal ${id}: 80 CULL yes, voting ends at ${endTs.toString()}`);
  await waitUntil(endTs.toNumber());
  await culling.executeProposal(id);
  const metadata = await getTokenMetadata(connection, brute, undefined, TOKEN_2022_PROGRAM_ID);
  console.log(`executed; Brute's URI is now ${metadata?.uri}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
