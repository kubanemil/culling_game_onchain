import { randomBytes } from "node:crypto";
import { AnchorProvider, BN, Program } from "@anchor-lang/core";
import { Keypair, PublicKey } from "@solana/web3.js";

import { commitment, newSecret } from "./duel.js";
import type { Asset } from "./idl/asset.js";
import type { Duel } from "./idl/duel.js";
import type { Governance } from "./idl/governance.js";
import type { Sale } from "./idl/sale.js";
import {
  ASSET_PROGRAM_ID,
  assetIdl,
  ata,
  duelIdl,
  GOVERNANCE_PROGRAM_ID,
  governanceIdl,
  pda,
  SALE_PROGRAM_ID,
  saleIdl,
} from "./programs.js";

export type CardField = "name" | "symbol" | "uri";

export interface Stats {
  attack: number;
  defense: number;
  health: number;
}

export interface RoundArgs {
  startTs: number;
  endTs: number;
  /** Lamports per whole token. */
  price: bigint;
  hardCap: bigint;
  merkleRoot: Buffer;
  cliff: number;
  vesting: number;
}

const CARD_FIELDS: CardField[] = ["name", "symbol", "uri"];

const bn = (n: bigint | number) => new BN(n.toString());
const bytes = (b: Buffer) => Array.from(b);
const field = (f: CardField) => ({ [f]: {} }) as never;
const fieldIndex = (encoded: object) =>
  CARD_FIELDS.indexOf(Object.keys(encoded)[0] as CardField);

/** The provider wallet pays fees and acts as admin. */
export class Culling {
  readonly sale: Program<Sale>;
  readonly asset: Program<Asset>;
  readonly duel: Program<Duel>;
  readonly governance: Program<Governance>;

  constructor(readonly provider: AnchorProvider) {
    this.sale = new Program(saleIdl as Sale, provider);
    this.asset = new Program(assetIdl as Asset, provider);
    this.duel = new Program(duelIdl as Duel, provider);
    this.governance = new Program(governanceIdl as Governance, provider);
  }

  get admin() {
    return this.provider.wallet.publicKey;
  }

  initializeSale(args: {
    decimals: number;
    supplyCap: bigint;
    name: string;
    symbol: string;
    uri: string;
  }) {
    return this.sale.methods
      .initialize({ ...args, supplyCap: bn(args.supplyCap) })
      .accountsPartial({
        admin: this.admin,
        config: pda.saleConfig(),
        mint: pda.tokenMint(),
        programData: pda.programData(SALE_PROGRAM_ID),
      })
      .rpc();
  }

  async createRound(args: RoundArgs): Promise<bigint> {
    const config = await this.sale.account.config.fetch(pda.saleConfig());
    const id = BigInt(config.roundCount.toString());
    await this.sale.methods
      .createRound({
        startTs: bn(args.startTs),
        endTs: bn(args.endTs),
        price: bn(args.price),
        hardCap: bn(args.hardCap),
        merkleRoot: bytes(args.merkleRoot),
        cliff: bn(args.cliff),
        vesting: bn(args.vesting),
      })
      .accountsPartial({ admin: this.admin, config: pda.saleConfig(), round: pda.round(id) })
      .rpc();
    return id;
  }

  buy(buyer: Keypair, round: bigint, amount: bigint, maxAmount: bigint, proof: Buffer[]) {
    const roundKey = pda.round(round);
    return this.sale.methods
      .buy(bn(amount), bn(maxAmount), proof.map(bytes))
      .accountsPartial({
        buyer: buyer.publicKey,
        config: pda.saleConfig(),
        round: roundKey,
        allocation: pda.allocation(roundKey, buyer.publicKey),
      })
      .signers([buyer])
      .rpc();
  }

  claim(buyer: Keypair, round: bigint) {
    const roundKey = pda.round(round);
    return this.sale.methods
      .claim()
      .accountsPartial({
        buyer: buyer.publicKey,
        config: pda.saleConfig(),
        mint: pda.tokenMint(),
        round: roundKey,
        allocation: pda.allocation(roundKey, buyer.publicKey),
        buyerTokens: ata(buyer.publicKey, pda.tokenMint()),
      })
      .signers([buyer])
      .rpc();
  }

  /** Anyone can call this after the round ends to give its unsold supply back. */
  finalizeRound(round: bigint) {
    return this.sale.methods
      .finalizeRound()
      .accountsPartial({ config: pda.saleConfig(), round: pda.round(round) })
      .rpc();
  }

  withdrawProceeds(round: bigint) {
    return this.sale.methods
      .withdrawProceeds()
      .accountsPartial({ admin: this.admin, config: pda.saleConfig(), round: pda.round(round) })
      .rpc();
  }

  initializeAsset() {
    return this.asset.methods
      .initialize()
      .accountsPartial({
        admin: this.admin,
        config: pda.assetConfig(),
        programData: pda.programData(ASSET_PROGRAM_ID),
      })
      .rpc();
  }

  async createCard(args: {
    stats: Stats;
    price: bigint;
    name: string;
    symbol: string;
    uri: string;
  }): Promise<PublicKey> {
    const config = await this.asset.account.config.fetch(pda.assetConfig());
    const mint = pda.cardMint(config.cardCount);
    await this.asset.methods
      .createCard({ ...args, price: bn(args.price) })
      .accountsPartial({
        admin: this.admin,
        config: pda.assetConfig(),
        mint,
        card: pda.card(mint),
      })
      .rpc();
    return mint;
  }

  buyCard(player: Keypair, cardMint: PublicKey) {
    const tokenMint = pda.tokenMint();
    return this.asset.methods
      .buyCard()
      .accountsPartial({
        buyer: player.publicKey,
        config: pda.assetConfig(),
        card: pda.card(cardMint),
        cardMint,
        tokenMint,
        buyerTokens: ata(player.publicKey, tokenMint),
        buyerCards: ata(player.publicKey, cardMint),
      })
      .signers([player])
      .rpc();
  }

  /** Keep the returned secret: without it the creator cannot reveal and forfeits. */
  async createDuel(creator: Keypair, cardMint: PublicKey, stake: bigint, opponent?: PublicKey) {
    const nonce = randomBytes(8).readBigUInt64LE();
    const duel = pda.duel(creator.publicKey, nonce);
    const secret = newSecret();
    await this.duel.methods
      .createDuel(
        bn(nonce),
        bn(stake),
        bytes(commitment(duel, creator.publicKey, cardMint, secret)),
        opponent ?? null,
      )
      .accountsPartial({ creator: creator.publicKey, duel })
      .signers([creator])
      .rpc();
    return { duel, secret };
  }

  async acceptDuel(opponent: Keypair, duel: PublicKey, cardMint: PublicKey) {
    const secret = newSecret();
    await this.duel.methods
      .acceptDuel(bytes(commitment(duel, opponent.publicKey, cardMint, secret)))
      .accountsPartial({ opponent: opponent.publicKey, duel })
      .signers([opponent])
      .rpc();
    return secret;
  }

  reveal(player: Keypair, duel: PublicKey, cardMint: PublicKey, secret: Buffer) {
    return this.duel.methods
      .reveal(bytes(secret))
      .accountsPartial({
        player: player.publicKey,
        duel,
        card: pda.card(cardMint),
        cardMint,
        playerCards: ata(player.publicKey, cardMint),
        escrow: pda.escrow(duel, player.publicKey),
      })
      .signers([player])
      .rpc();
  }

  cancelDuel(creator: Keypair, duel: PublicKey) {
    return this.duel.methods
      .cancelDuel()
      .accountsPartial({ creator: creator.publicKey, duel })
      .signers([creator])
      .rpc();
  }

  /** Works once both players revealed or the reveal window closed. */
  async claimDuel(player: Keypair, duel: PublicKey) {
    const state = await this.duel.account.duel.fetch(duel);
    const revealed = player.publicKey.equals(state.creator)
      ? state.creatorReveal
      : state.opponentReveal;
    const card = revealed?.card ?? null;
    return this.duel.methods
      .claim()
      .accountsPartial({
        player: player.publicKey,
        duel,
        cardMint: card,
        escrow: card && pda.escrow(duel, player.publicKey),
        playerCards: card && ata(player.publicKey, card),
      })
      .signers([player])
      .rpc();
  }

  initializeGovernance(args: { quorum: bigint; thresholdBps: number; votingPeriod: number }) {
    return this.governance.methods
      .initialize({
        quorum: bn(args.quorum),
        thresholdBps: args.thresholdBps,
        votingPeriod: bn(args.votingPeriod),
      })
      .accountsPartial({
        admin: this.admin,
        config: pda.governanceConfig(),
        mint: pda.tokenMint(),
        programData: pda.programData(GOVERNANCE_PROGRAM_ID),
      })
      .rpc();
  }

  deposit(owner: Keypair, amount: bigint) {
    const mint = pda.tokenMint();
    const voter = pda.voter(owner.publicKey);
    return this.governance.methods
      .deposit(bn(amount))
      .accountsPartial({
        owner: owner.publicKey,
        config: pda.governanceConfig(),
        mint,
        voter,
        escrow: ata(voter, mint),
        ownerTokens: ata(owner.publicKey, mint),
      })
      .signers([owner])
      .rpc();
  }

  withdraw(owner: Keypair, amount: bigint) {
    const mint = pda.tokenMint();
    const voter = pda.voter(owner.publicKey);
    return this.governance.methods
      .withdraw(bn(amount))
      .accountsPartial({
        owner: owner.publicKey,
        config: pda.governanceConfig(),
        mint,
        voter,
        escrow: ata(voter, mint),
        ownerTokens: ata(owner.publicKey, mint),
      })
      .signers([owner])
      .rpc();
  }

  async createProposal(
    proposer: Keypair,
    cardMint: PublicKey,
    cardField: CardField,
    value: string,
  ): Promise<bigint> {
    const config = await this.governance.account.config.fetch(pda.governanceConfig());
    const id = BigInt(config.proposalCount.toString());
    await this.governance.methods
      .createProposal(field(cardField), value)
      .accountsPartial({
        proposer: proposer.publicKey,
        config: pda.governanceConfig(),
        card: pda.card(cardMint),
        proposal: pda.proposal(id),
      })
      .signers([proposer])
      .rpc();
    return id;
  }

  vote(owner: Keypair, id: bigint, approve: boolean) {
    const mint = pda.tokenMint();
    const proposal = pda.proposal(id);
    const voter = pda.voter(owner.publicKey);
    return this.governance.methods
      .vote(approve)
      .accountsPartial({
        owner: owner.publicKey,
        config: pda.governanceConfig(),
        mint,
        proposal,
        voter,
        escrow: ata(voter, mint),
        vote: pda.voteRecord(proposal, owner.publicKey),
      })
      .signers([owner])
      .rpc();
  }

  async executeProposal(id: bigint) {
    const proposal = pda.proposal(id);
    const state = await this.governance.account.proposal.fetch(proposal);
    return this.governance.methods
      .executeProposal()
      .accountsPartial({
        payer: this.admin,
        config: pda.governanceConfig(),
        proposal,
        fieldVersion: pda.fieldVersion(state.cardMint, fieldIndex(state.field)),
        authority: pda.governanceAuthority(),
        assetConfig: pda.assetConfig(),
        card: pda.card(state.cardMint),
        cardMint: state.cardMint,
      })
      .rpc();
  }
}
