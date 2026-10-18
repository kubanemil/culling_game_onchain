//! A deployed economy: token sale, three cards and governance, plus helpers to fund players.

use ::asset::{CreateCardArgs, Stats};
use ::governance::InitializeArgs as GovernanceArgs;
use ::sale::{merkle, InitializeArgs as SaleArgs, RoundArgs, TOKEN_MINT};
use anchor_lang::prelude::Pubkey;
use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::{
    harness::{ata, Env},
    ix,
};

pub const DECIMALS: u8 = 6;
pub const TOKEN: u64 = 1_000_000;
pub const SUPPLY_CAP: u64 = 1_000_000 * TOKEN;
/// 0.002 SOL per whole token.
pub const PRICE: u64 = 2_000_000;
pub const CARD_PRICE: u64 = 10 * TOKEN;
pub const QUORUM: u64 = 100 * TOKEN;
pub const VOTING_PERIOD: i64 = 3 * 24 * 60 * 60;

pub const BRUTE: Stats = Stats {
    attack: 14,
    defense: 2,
    health: 40,
};
pub const GUARD: Stats = Stats {
    attack: 6,
    defense: 9,
    health: 55,
};
pub const GLASS: Stats = Stats {
    attack: 20,
    defense: 0,
    health: 24,
};

pub struct World {
    pub env: Env,
    pub cards: Vec<Pubkey>,
    rounds: u64,
}

impl World {
    pub fn new() -> Self {
        let mut env = Env::new();
        let admin = env.deployer.insecure_clone();
        env.ok(
            &[ix::sale::initialize(&admin.pubkey(), sale_args())],
            &[&admin],
        );
        env.ok(&[ix::asset::initialize(&admin.pubkey())], &[&admin]);
        env.ok(
            &[ix::governance::initialize(
                &admin.pubkey(),
                GovernanceArgs {
                    quorum: QUORUM,
                    threshold_bps: 5_000,
                    voting_period: VOTING_PERIOD,
                },
            )],
            &[&admin],
        );

        let cards = [("Brute", BRUTE), ("Guard", GUARD), ("Glass", GLASS)]
            .into_iter()
            .enumerate()
            .map(|(id, (name, stats))| {
                let id = id as u16;
                env.ok(
                    &[ix::asset::create_card(
                        &admin.pubkey(),
                        id,
                        card_args(name, stats),
                    )],
                    &[&admin],
                );
                ix::asset::card_mint(id)
            })
            .collect();

        Self {
            env,
            cards,
            rounds: 0,
        }
    }

    pub fn admin(&self) -> Keypair {
        self.env.deployer.insecure_clone()
    }

    pub fn round_args(&self, merkle_root: [u8; 32], hard_cap: u64) -> RoundArgs {
        let now = self.env.now();
        RoundArgs {
            start_ts: now,
            end_ts: now + 1_000,
            price: PRICE,
            hard_cap,
            merkle_root,
            cliff: 0,
            vesting: 0,
        }
    }

    /// Creates a round and returns its id.
    pub fn create_round(&mut self, args: RoundArgs) -> u64 {
        let admin = self.admin();
        let id = self.rounds;
        self.env.ok(
            &[ix::sale::create_round(&admin.pubkey(), id, args)],
            &[&admin],
        );
        self.rounds += 1;
        id
    }

    /// Players who bought `tokens` each in a fully unlocked round and claimed them.
    pub fn funded_players(&mut self, count: usize, tokens: u64) -> Vec<Keypair> {
        let players: Vec<Keypair> = (0..count).map(|_| self.env.user()).collect();
        let leaves: Vec<[u8; 32]> = players
            .iter()
            .map(|p| merkle::leaf(&p.pubkey(), tokens))
            .collect();
        let (root, proofs) = merkle::build(&leaves);
        let args = self.round_args(root, tokens * count as u64);
        let end = args.end_ts;
        let round = self.create_round(args);
        for (player, proof) in players.iter().zip(proofs) {
            self.env.ok(
                &[ix::sale::buy(
                    &player.pubkey(),
                    round,
                    tokens,
                    tokens,
                    proof,
                )],
                &[player],
            );
        }
        self.env.warp_to(end);
        for player in &players {
            self.env
                .ok(&[ix::sale::claim(&player.pubkey(), round)], &[player]);
        }
        players
    }

    pub fn buy_card(&mut self, player: &Keypair, card: usize) {
        let mint = self.cards[card];
        self.env
            .ok(&[ix::asset::buy_card(&player.pubkey(), &mint)], &[player]);
    }

    pub fn tokens_of(&self, owner: &Pubkey) -> u64 {
        self.env.token_balance(&ata(owner, &TOKEN_MINT))
    }
}

pub fn sale_args() -> SaleArgs {
    SaleArgs {
        decimals: DECIMALS,
        supply_cap: SUPPLY_CAP,
        name: "Culling Token".into(),
        symbol: "CULL".into(),
        uri: "https://example.com/cull.json".into(),
    }
}

pub fn card_args(name: &str, stats: Stats) -> CreateCardArgs {
    CreateCardArgs {
        stats,
        price: CARD_PRICE,
        name: name.into(),
        symbol: "CARD".into(),
        uri: format!("https://example.com/cards/{}.json", name.to_lowercase()),
    }
}
