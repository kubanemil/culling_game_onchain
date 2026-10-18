use std::slice;

use ::sale::{error::SaleError, merkle, Allocation, Config, Round, RoundArgs, TOKEN_MINT};
use anchor_lang::prelude::Pubkey;
use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::{
    harness::{expect_error, Env, TxResult},
    ix,
    world::{sale_args, World, DECIMALS, PRICE, SUPPLY_CAP, TOKEN},
};

const DAY: i64 = 24 * 60 * 60;

type RoundEdit = fn(&mut RoundArgs);

/// A world with one round whose allowlist gives each buyer `allocation`.
struct Sale {
    world: World,
    buyers: Vec<Keypair>,
    proofs: Vec<Vec<[u8; 32]>>,
    allocation: u64,
    round: u64,
}

impl Sale {
    fn new(buyers: usize, allocation: u64, configure: impl FnOnce(&mut RoundArgs)) -> Self {
        let mut world = World::new();
        let buyers: Vec<Keypair> = (0..buyers).map(|_| world.env.user()).collect();
        let leaves: Vec<_> = buyers
            .iter()
            .map(|b| merkle::leaf(&b.pubkey(), allocation))
            .collect();
        let (root, proofs) = merkle::build(&leaves);
        let mut args = world.round_args(root, allocation * buyers.len() as u64);
        configure(&mut args);
        let round = world.create_round(args);
        Self {
            world,
            buyers,
            proofs,
            allocation,
            round,
        }
    }

    fn buy(&mut self, buyer: usize, amount: u64) -> TxResult {
        let ix = ix::sale::buy(
            &self.buyers[buyer].pubkey(),
            self.round,
            amount,
            self.allocation,
            self.proofs[buyer].clone(),
        );
        self.world.env.send(&[ix], &[&self.buyers[buyer]])
    }

    fn claim(&mut self, buyer: usize) -> TxResult {
        let ix = ix::sale::claim(&self.buyers[buyer].pubkey(), self.round);
        self.world.env.send(&[ix], &[&self.buyers[buyer]])
    }

    fn round(&self) -> Round {
        self.world.env.account(&ix::sale::round(self.round))
    }

    fn allocation_of(&self, buyer: usize) -> Pubkey {
        ix::sale::allocation(&ix::sale::round(self.round), &self.buyers[buyer].pubkey())
    }
}

#[test]
fn initialize_creates_the_token_with_metadata() {
    let world = World::new();
    let config: Config = world.env.account(&ix::sale::config());
    assert_eq!(config.admin, world.env.deployer.pubkey());
    assert_eq!(config.mint, TOKEN_MINT);
    assert_eq!(config.decimals, DECIMALS);
    assert_eq!(config.supply_cap, SUPPLY_CAP);

    let metadata = world.env.metadata(&TOKEN_MINT);
    assert_eq!(metadata.symbol, "CULL");
    assert_eq!(metadata.mint, TOKEN_MINT);
    assert_eq!(
        Option::<Pubkey>::from(metadata.update_authority),
        Some(ix::sale::config())
    );
    assert_eq!(world.env.mint_supply(&TOKEN_MINT), 0);
}

#[test]
fn attack_only_the_upgrade_authority_can_initialize() {
    let mut env = Env::new();
    let stranger = env.user();
    let result = env.send(
        &[ix::sale::initialize(&stranger.pubkey(), sale_args())],
        &[&stranger],
    );
    expect_error(result, SaleError::NotUpgradeAuthority);
}

#[test]
fn admin_transfer_takes_two_steps() {
    let mut world = World::new();
    let admin = world.admin();
    let next = world.env.user();
    let stranger = world.env.user();

    world.env.ok(
        &[ix::sale::propose_admin(&admin.pubkey(), next.pubkey())],
        &[&admin],
    );
    let result = world
        .env
        .send(&[ix::sale::accept_admin(&stranger.pubkey())], &[&stranger]);
    expect_error(result, SaleError::NotPendingAdmin);

    world
        .env
        .ok(&[ix::sale::accept_admin(&next.pubkey())], &[&next]);
    let config: Config = world.env.account(&ix::sale::config());
    assert_eq!(config.admin, next.pubkey());
    assert_eq!(config.pending_admin, None);

    let args = world.round_args([0; 32], TOKEN);
    let result = world.env.send(
        &[ix::sale::create_round(&admin.pubkey(), 0, args.clone())],
        &[&admin],
    );
    expect_error(result, SaleError::Unauthorized);
    world
        .env
        .ok(&[ix::sale::create_round(&next.pubkey(), 0, args)], &[&next]);
}

#[test]
fn attack_a_non_admin_cannot_create_or_pause_rounds() {
    let mut sale = Sale::new(1, TOKEN, |_| {});
    let stranger = sale.world.env.user();
    let args = sale.world.round_args([0; 32], TOKEN);

    let result = sale.world.env.send(
        &[ix::sale::create_round(&stranger.pubkey(), 1, args)],
        &[&stranger],
    );
    expect_error(result, SaleError::Unauthorized);

    let result = sale.world.env.send(
        &[ix::sale::set_paused(&stranger.pubkey(), sale.round, true)],
        &[&stranger],
    );
    expect_error(result, SaleError::Unauthorized);

    let result = sale.world.env.send(
        &[ix::sale::propose_admin(
            &stranger.pubkey(),
            stranger.pubkey(),
        )],
        &[&stranger],
    );
    expect_error(result, SaleError::Unauthorized);
}

#[test]
fn round_caps_can_never_exceed_the_supply_cap() {
    let mut world = World::new();
    let admin = world.admin();
    let args = world.round_args([0; 32], SUPPLY_CAP - TOKEN);
    world.create_round(args);

    let args = world.round_args([0; 32], 2 * TOKEN);
    let result = world.env.send(
        &[ix::sale::create_round(&admin.pubkey(), 1, args)],
        &[&admin],
    );
    expect_error(result, SaleError::SupplyCapExceeded);
}

#[test]
fn round_parameters_are_validated() {
    let mut world = World::new();
    let admin = world.admin();
    let base = world.round_args([0; 32], TOKEN);
    let cases: [(RoundEdit, SaleError); 5] = [
        (|a| a.end_ts = a.start_ts, SaleError::InvalidSchedule),
        (
            |a| a.end_ts = a.start_ts - 10 * DAY,
            SaleError::InvalidSchedule,
        ),
        (|a| a.price = 0, SaleError::InvalidPricing),
        (|a| a.cliff = a.vesting + 1, SaleError::InvalidVesting),
        (|a| a.vesting = 11 * 365 * DAY, SaleError::InvalidVesting),
    ];
    for (mutate, error) in cases {
        let mut args = base.clone();
        mutate(&mut args);
        let result = world.env.send(
            &[ix::sale::create_round(&admin.pubkey(), 0, args)],
            &[&admin],
        );
        expect_error(result, error);
    }
}

#[test]
fn buy_charges_a_decimals_correct_price_into_the_round() {
    let mut sale = Sale::new(2, 10 * TOKEN, |_| {});
    let round_key = ix::sale::round(sale.round);
    let before = sale.world.env.lamports(&round_key);

    sale.buy(0, 5 * TOKEN / 2).unwrap();

    let cost = 5 * PRICE / 2;
    assert_eq!(sale.world.env.lamports(&round_key) - before, cost);
    let round = sale.round();
    assert_eq!(round.sold, 5 * TOKEN / 2);
    assert_eq!(round.proceeds, cost);
    let allocation: Allocation = sale.world.env.account(&sale.allocation_of(0));
    assert_eq!(allocation.purchased, 5 * TOKEN / 2);
    assert_eq!(allocation.max_amount, 10 * TOKEN);
}

#[test]
fn attack_a_wallet_outside_the_allowlist_cannot_buy() {
    let mut sale = Sale::new(3, TOKEN, |_| {});
    let outsider = sale.world.env.user();
    let ix = ix::sale::buy(
        &outsider.pubkey(),
        sale.round,
        TOKEN,
        TOKEN,
        sale.proofs[0].clone(),
    );
    expect_error(
        sale.world.env.send(&[ix], &[&outsider]),
        SaleError::InvalidProof,
    );
}

#[test]
fn attack_a_buyer_cannot_claim_a_bigger_allocation_than_listed() {
    let mut sale = Sale::new(3, TOKEN, |_| {});
    let buyer = sale.buyers[1].insecure_clone();
    let ix = ix::sale::buy(
        &buyer.pubkey(),
        sale.round,
        2 * TOKEN,
        2 * TOKEN,
        sale.proofs[1].clone(),
    );
    expect_error(
        sale.world.env.send(&[ix], &[&buyer]),
        SaleError::InvalidProof,
    );
}

#[test]
fn attack_a_buyer_cannot_exceed_allocation_by_moving_tokens() {
    let mut sale = Sale::new(2, 10 * TOKEN, |_| {});
    sale.buy(0, 6 * TOKEN).unwrap();
    sale.buy(0, 4 * TOKEN).unwrap();

    // Purchases vest in the program, so the wallet holds nothing to move out.
    assert_eq!(sale.world.tokens_of(&sale.buyers[0].pubkey()), 0);
    expect_error(sale.buy(0, 1), SaleError::AllocationExceeded);
}

#[test]
fn attack_the_hard_cap_holds_across_buyers() {
    let mut sale = Sale::new(3, 10 * TOKEN, |a| a.hard_cap = 15 * TOKEN);
    sale.buy(0, 10 * TOKEN).unwrap();
    sale.buy(1, 5 * TOKEN).unwrap();
    expect_error(sale.buy(2, 1), SaleError::HardCapExceeded);
    assert_eq!(sale.round().sold, 15 * TOKEN);
}

#[test]
fn buying_is_blocked_while_paused_and_outside_the_window() {
    let mut sale = Sale::new(1, 10 * TOKEN, |a| {
        a.start_ts += 100;
        a.end_ts = a.start_ts + 100;
    });
    expect_error(sale.buy(0, TOKEN), SaleError::RoundNotStarted);

    sale.world.env.warp(100);
    let admin = sale.world.admin();
    sale.world.env.ok(
        &[ix::sale::set_paused(&admin.pubkey(), sale.round, true)],
        &[&admin],
    );
    expect_error(sale.buy(0, TOKEN), SaleError::RoundPaused);

    sale.world.env.ok(
        &[ix::sale::set_paused(&admin.pubkey(), sale.round, false)],
        &[&admin],
    );
    sale.buy(0, TOKEN).unwrap();
    expect_error(sale.buy(0, 0), SaleError::ZeroAmount);

    sale.world.env.warp(100);
    expect_error(sale.buy(0, TOKEN), SaleError::RoundEnded);
}

#[test]
fn claims_follow_the_cliff_and_linear_vesting_then_close_the_allocation() {
    let mut sale = Sale::new(1, 100 * TOKEN, |a| {
        a.cliff = 30 * DAY;
        a.vesting = 100 * DAY;
    });
    sale.buy(0, 100 * TOKEN).unwrap();
    let end = sale.round().end_ts;
    let buyer = sale.buyers[0].pubkey();

    expect_error(sale.claim(0), SaleError::NothingToClaim);

    sale.world.env.warp_to(end + 30 * DAY - 1);
    expect_error(sale.claim(0), SaleError::NothingToClaim);

    sale.world.env.warp_to(end + 30 * DAY);
    sale.claim(0).unwrap();
    assert_eq!(sale.world.tokens_of(&buyer), 30 * TOKEN);

    sale.world.env.warp_to(end + 75 * DAY);
    sale.claim(0).unwrap();
    assert_eq!(sale.world.tokens_of(&buyer), 75 * TOKEN);

    let allocation = sale.allocation_of(0);
    let rent = sale.world.env.lamports(&allocation);
    let before = sale.world.env.lamports(&buyer);
    sale.world.env.warp_to(end + 200 * DAY);
    sale.claim(0).unwrap();
    assert_eq!(sale.world.tokens_of(&buyer), 100 * TOKEN);
    assert_eq!(sale.world.env.mint_supply(&TOKEN_MINT), 100 * TOKEN);
    assert!(!sale.world.env.exists(&allocation));
    assert_eq!(sale.world.env.lamports(&buyer), before + rent - 5_000);
}

#[test]
fn attack_a_buyer_cannot_claim_the_same_tokens_twice() {
    let mut sale = Sale::new(1, 100 * TOKEN, |a| a.vesting = 100 * DAY);
    sale.buy(0, 100 * TOKEN).unwrap();
    let end = sale.round().end_ts;

    sale.world.env.warp_to(end + 10 * DAY);
    sale.claim(0).unwrap();
    expect_error(sale.claim(0), SaleError::NothingToClaim);
    assert_eq!(sale.world.tokens_of(&sale.buyers[0].pubkey()), 10 * TOKEN);
}

#[test]
fn attack_a_buyer_cannot_claim_someone_elses_allocation() {
    let mut sale = Sale::new(2, 10 * TOKEN, |_| {});
    sale.buy(0, 10 * TOKEN).unwrap();
    sale.world.env.warp(1_000);

    let thief = sale.buyers[1].insecure_clone();
    let ix = ix::sale::claim_with(&thief.pubkey(), sale.round, sale.allocation_of(0));
    expect_error(
        sale.world.env.send(&[ix], &[&thief]),
        anchor_lang::error::ErrorCode::ConstraintSeeds,
    );
}

#[test]
fn attack_a_non_admin_cannot_withdraw_proceeds() {
    let mut sale = Sale::new(1, 10 * TOKEN, |_| {});
    sale.buy(0, 10 * TOKEN).unwrap();
    sale.world.env.warp(1_000);

    let stranger = sale.world.env.user();
    let result = sale.world.env.send(
        &[ix::sale::withdraw_proceeds(&stranger.pubkey(), sale.round)],
        &[&stranger],
    );
    expect_error(result, SaleError::Unauthorized);
}

#[test]
fn admin_withdraws_proceeds_once_and_only_after_the_round_ends() {
    let mut sale = Sale::new(1, 10 * TOKEN, |_| {});
    sale.buy(0, 10 * TOKEN).unwrap();
    let admin = sale.world.admin();
    let withdraw = ix::sale::withdraw_proceeds(&admin.pubkey(), sale.round);

    expect_error(
        sale.world.env.send(slice::from_ref(&withdraw), &[&admin]),
        SaleError::RoundNotEnded,
    );

    sale.world.env.warp(1_000);
    let before = sale.world.env.lamports(&admin.pubkey());
    sale.world.env.ok(slice::from_ref(&withdraw), &[&admin]);
    assert_eq!(
        sale.world.env.lamports(&admin.pubkey()),
        before + 10 * PRICE - 5_000
    );
    assert_eq!(sale.round().proceeds, 0);

    expect_error(
        sale.world.env.send(&[withdraw], &[&admin]),
        SaleError::NoProceeds,
    );
}
