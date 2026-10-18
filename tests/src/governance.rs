use std::slice;

use ::asset::{CardField, MAX_URI_LEN};
use ::governance::{
    error::GovernanceError, FieldVersion, InitializeArgs, Proposal, ProposalStatus, VoteRecord,
    Voter, AUTHORITY, EXECUTION_WINDOW,
};
use ::sale::TOKEN_MINT;
use anchor_lang::{error::ErrorCode, prelude::Pubkey};
use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::{
    harness::{ata, expect_error, expect_log, Env, TxResult},
    ix,
    world::{sale_args, World, DECIMALS, QUORUM, TOKEN, VOTING_PERIOD},
};

const NEW_URI: &str = "https://example.com/cards/brute-v2.json";

/// Alice holds 150 tokens, Bob 100, Carol 1. Each has deposited everything.
struct Dao {
    world: World,
    alice: Keypair,
    bob: Keypair,
    carol: Keypair,
}

impl Dao {
    fn new() -> Self {
        let mut world = World::new();
        let alice = world.funded_players(1, 150 * TOKEN).remove(0);
        let bob = world.funded_players(1, 100 * TOKEN).remove(0);
        let carol = world.funded_players(1, 1).remove(0);
        let mut dao = Self {
            world,
            alice,
            bob,
            carol,
        };
        for (voter, amount) in [(0, 150 * TOKEN), (1, 100 * TOKEN), (2, 1)] {
            let voter = dao.voter(voter);
            dao.world.env.ok(
                &[ix::governance::deposit(&voter.pubkey(), amount)],
                &[&voter],
            );
        }
        dao
    }

    fn voter(&self, index: usize) -> Keypair {
        [&self.alice, &self.bob, &self.carol][index].insecure_clone()
    }

    fn propose(&mut self, id: u64, field: CardField, value: &str) -> TxResult {
        let card = self.world.cards[0];
        let proposer = self.voter(2);
        let ix = ix::governance::create_proposal(&proposer.pubkey(), id, &card, field, value);
        self.world.env.send(&[ix], &[&proposer])
    }

    fn vote(&mut self, voter: usize, id: u64, approve: bool) -> TxResult {
        let voter = self.voter(voter);
        let ix = ix::governance::vote(&voter.pubkey(), id, approve);
        self.world.env.send(&[ix], &[&voter])
    }

    fn execute(&mut self, id: u64) -> TxResult {
        let card = self.world.cards[0];
        let field = self.proposal(id).field;
        let payer = self.voter(2);
        self.world.env.send(
            &[ix::governance::execute(&payer.pubkey(), id, &card, field)],
            &[&payer],
        )
    }

    fn proposal(&self, id: u64) -> Proposal {
        self.world.env.account(&ix::governance::proposal(id))
    }

    fn end_voting(&mut self, id: u64) {
        let end = self.proposal(id).end_ts;
        self.world.env.warp_to(end);
    }
}

#[test]
fn program_ids_are_consistent() {
    assert_eq!(::asset::GOVERNANCE_PROGRAM_ID, ::governance::ID);
    assert_eq!(::asset::GOVERNANCE_AUTHORITY, AUTHORITY);
    assert_eq!(
        TOKEN_MINT,
        Pubkey::find_program_address(&[::sale::MINT_SEED], &::sale::ID).0
    );
}

#[test]
fn a_passed_proposal_updates_card_metadata_exactly_once() {
    let mut dao = Dao::new();
    let mint = dao.world.cards[0];
    let long_uri = format!("https://example.com/{}", "x".repeat(MAX_URI_LEN - 20));
    assert_eq!(long_uri.len(), MAX_URI_LEN);

    dao.propose(0, CardField::Uri, &long_uri).unwrap();
    dao.vote(0, 0, true).unwrap();
    dao.vote(1, 0, false).unwrap();
    expect_error(dao.execute(0), GovernanceError::VotingNotEnded);

    dao.end_voting(0);
    dao.execute(0).unwrap();
    assert_eq!(dao.world.env.metadata(&mint).uri, long_uri);
    let proposal = dao.proposal(0);
    assert_eq!(proposal.status, ProposalStatus::Executed);
    assert_eq!((proposal.yes, proposal.no), (150 * TOKEN, 100 * TOKEN));

    let account = dao.world.env.svm.get_account(&mint).unwrap();
    let rent = dao
        .world
        .env
        .svm
        .minimum_balance_for_rent_exemption(account.data.len());
    assert!(account.lamports >= rent);

    expect_error(dao.execute(0), GovernanceError::AlreadyExecuted);
}

#[test]
fn attack_a_voter_cannot_vote_twice() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    dao.vote(1, 0, true).unwrap();
    expect_log(dao.vote(1, 0, true), "already in use");
    expect_log(dao.vote(1, 0, false), "already in use");
    assert_eq!(dao.proposal(0).yes, 100 * TOKEN);
    assert_eq!(dao.proposal(0).no, 0);
}

#[test]
fn attack_a_self_minted_token_has_no_voting_power() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    let carol = dao.voter(2);
    let fake = dao.world.env.fake_mint(&carol, DECIMALS);
    dao.world
        .env
        .mint_to(&carol, &fake, &carol.pubkey(), 1_000_000 * TOKEN);

    let result = dao.world.env.send(
        &[ix::governance::deposit_with(
            &carol.pubkey(),
            1_000_000 * TOKEN,
            &fake,
        )],
        &[&carol],
    );
    expect_error(result, ErrorCode::ConstraintHasOne);

    // Even fake tokens sitting in an account owned by Carol's voter PDA do not count.
    let voter_pda = ix::governance::voter(&carol.pubkey());
    dao.world
        .env
        .mint_to(&carol, &fake, &voter_pda, 1_000_000 * TOKEN);
    let result = dao.world.env.send(
        &[ix::governance::vote_with(&carol.pubkey(), 0, true, &fake)],
        &[&carol],
    );
    expect_error(result, ErrorCode::ConstraintHasOne);

    dao.vote(2, 0, true).unwrap();
    assert_eq!(dao.proposal(0).yes, 1);
}

#[test]
fn attack_tokens_stay_locked_until_the_vote_ends_so_they_cannot_vote_again() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    dao.vote(1, 0, true).unwrap();
    let bob = dao.voter(1);
    let withdraw = ix::governance::withdraw(&bob.pubkey(), 100 * TOKEN);

    expect_error(
        dao.world.env.send(slice::from_ref(&withdraw), &[&bob]),
        GovernanceError::TokensLocked,
    );

    dao.end_voting(0);
    dao.world.env.ok(&[withdraw], &[&bob]);
    assert_eq!(dao.world.tokens_of(&bob.pubkey()), 100 * TOKEN);
    assert_eq!(
        dao.world
            .env
            .token_balance(&ix::governance::escrow(&bob.pubkey())),
        0
    );
    let voter: Voter = dao.world.env.account(&ix::governance::voter(&bob.pubkey()));
    assert_eq!(voter.locked_until, dao.proposal(0).end_ts);
}

#[test]
fn tokens_deposited_after_a_vote_stay_withdrawable() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    let alice = dao.voter(0);
    let withdraw = |amount| ix::governance::withdraw(&alice.pubkey(), amount);

    dao.world.env.ok(&[withdraw(50 * TOKEN)], &[&alice]);
    dao.vote(0, 0, true).unwrap();
    dao.world.env.ok(
        &[ix::governance::deposit(&alice.pubkey(), 50 * TOKEN)],
        &[&alice],
    );
    let voter: Voter = dao
        .world
        .env
        .account(&ix::governance::voter(&alice.pubkey()));
    assert_eq!(voter.locked_amount, 100 * TOKEN);

    dao.world.env.ok(&[withdraw(50 * TOKEN)], &[&alice]);
    expect_error(
        dao.world.env.send(&[withdraw(1)], &[&alice]),
        GovernanceError::TokensLocked,
    );
    assert_eq!(dao.proposal(0).yes, 100 * TOKEN);
}

#[test]
fn attack_an_old_passed_proposal_cannot_undo_a_newer_change() {
    let mut dao = Dao::new();
    let mint = dao.world.cards[0];
    dao.propose(0, CardField::Uri, "https://example.com/old.json")
        .unwrap();
    dao.propose(1, CardField::Uri, NEW_URI).unwrap();
    dao.propose(2, CardField::Name, "Brute II").unwrap();
    for id in 0..3 {
        dao.vote(0, id, true).unwrap();
    }
    dao.end_voting(2);

    dao.execute(1).unwrap();
    expect_error(dao.execute(0), GovernanceError::StaleProposal);
    assert_eq!(dao.world.env.metadata(&mint).uri, NEW_URI);

    dao.execute(2).unwrap();
    let version: FieldVersion = dao
        .world
        .env
        .account(&ix::governance::field_version(&mint, CardField::Uri));
    assert_eq!(version.last_proposal, Some(1));
}

#[test]
fn attack_a_passed_proposal_expires_after_the_execution_window() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    dao.vote(0, 0, true).unwrap();
    let end = dao.proposal(0).end_ts;
    dao.world.env.warp_to(end + EXECUTION_WINDOW);
    expect_error(dao.execute(0), GovernanceError::ProposalExpired);
    assert_ne!(dao.world.env.metadata(&dao.world.cards[0]).uri, NEW_URI);
}

#[test]
fn attack_a_proposal_without_quorum_or_majority_cannot_execute() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    dao.propose(1, CardField::Uri, NEW_URI).unwrap();

    dao.vote(2, 0, true).unwrap();
    dao.vote(1, 1, true).unwrap();
    dao.vote(0, 1, false).unwrap();
    dao.end_voting(1);

    assert!(dao.proposal(0).yes < QUORUM);
    expect_error(dao.execute(0), GovernanceError::QuorumNotReached);
    expect_error(dao.execute(1), GovernanceError::ThresholdNotReached);
    assert_eq!(dao.proposal(1).status, ProposalStatus::Voting);
}

#[test]
fn attack_votes_after_the_deadline_are_rejected() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    dao.end_voting(0);
    expect_error(dao.vote(0, 0, true), GovernanceError::VotingEnded);
}

#[test]
fn attack_a_passed_proposal_cannot_be_pointed_at_another_card() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Name, "Brute II").unwrap();
    dao.vote(0, 0, true).unwrap();
    dao.end_voting(0);

    let other = dao.world.cards[1];
    let payer = dao.voter(2);
    let result = dao.world.env.send(
        &[ix::governance::execute(
            &payer.pubkey(),
            0,
            &other,
            CardField::Name,
        )],
        &[&payer],
    );
    expect_error(result, ErrorCode::ConstraintHasOne);
    assert_eq!(dao.world.env.metadata(&other).name, "Guard");
}

#[test]
fn one_wallet_can_open_many_proposals() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    dao.propose(1, CardField::Name, "Brute II").unwrap();
    dao.propose(2, CardField::Symbol, "BRT").unwrap();
    assert_eq!(dao.proposal(1).id, 1);
    assert_eq!(dao.proposal(2).field, CardField::Symbol);
    assert_eq!(
        dao.proposal(2).end_ts - dao.proposal(2).start_ts,
        VOTING_PERIOD
    );
}

#[test]
fn proposal_values_are_validated() {
    let mut dao = Dao::new();
    expect_error(
        dao.propose(0, CardField::Uri, &"x".repeat(MAX_URI_LEN + 1)),
        GovernanceError::InvalidValue,
    );
    expect_error(
        dao.propose(0, CardField::Name, ""),
        GovernanceError::InvalidValue,
    );
    expect_error(
        dao.propose(0, CardField::Symbol, "TOOLONGSYMBOL"),
        GovernanceError::InvalidValue,
    );
}

#[test]
fn a_vote_record_can_be_closed_for_its_rent_only_after_voting() {
    let mut dao = Dao::new();
    dao.propose(0, CardField::Uri, NEW_URI).unwrap();
    dao.vote(0, 0, true).unwrap();
    let alice = dao.voter(0);
    let record = ix::governance::vote_record(&ix::governance::proposal(0), &alice.pubkey());
    let saved: VoteRecord = dao.world.env.account(&record);
    assert_eq!(saved.weight, 150 * TOKEN);

    let close = ix::governance::close_vote(&alice.pubkey(), 0);
    expect_error(
        dao.world.env.send(slice::from_ref(&close), &[&alice]),
        GovernanceError::VotingNotEnded,
    );

    dao.end_voting(0);
    let rent = dao.world.env.lamports(&record);
    let before = dao.world.env.lamports(&alice.pubkey());
    dao.world.env.ok(&[close], &[&alice]);
    assert!(!dao.world.env.exists(&record));
    assert_eq!(
        dao.world.env.lamports(&alice.pubkey()),
        before + rent - 5_000
    );
    expect_error(dao.vote(0, 0, true), GovernanceError::VotingEnded);
}

#[test]
fn deposits_sit_in_escrow_and_zero_is_rejected() {
    let mut dao = Dao::new();
    let alice = dao.voter(0);
    let result = dao
        .world
        .env
        .send(&[ix::governance::deposit(&alice.pubkey(), 0)], &[&alice]);
    expect_error(result, GovernanceError::ZeroAmount);
    assert_eq!(
        dao.world
            .env
            .token_balance(&ix::governance::escrow(&alice.pubkey())),
        150 * TOKEN
    );
    assert_eq!(
        dao.world
            .env
            .token_balance(&ata(&alice.pubkey(), &TOKEN_MINT)),
        0
    );
}

#[test]
fn attack_only_the_upgrade_authority_can_initialize_governance() {
    let mut env = Env::new();
    let admin = env.deployer.insecure_clone();
    env.ok(
        &[ix::sale::initialize(&admin.pubkey(), sale_args())],
        &[&admin],
    );
    let stranger = env.user();
    let args = InitializeArgs {
        quorum: QUORUM,
        threshold_bps: 5_000,
        voting_period: VOTING_PERIOD,
    };
    let result = env.send(
        &[ix::governance::initialize(&stranger.pubkey(), args.clone())],
        &[&stranger],
    );
    expect_error(result, GovernanceError::NotUpgradeAuthority);

    let cases = [
        (
            InitializeArgs {
                quorum: 0,
                ..args.clone()
            },
            GovernanceError::InvalidQuorum,
        ),
        (
            InitializeArgs {
                threshold_bps: 4_999,
                ..args.clone()
            },
            GovernanceError::InvalidThreshold,
        ),
        (
            InitializeArgs {
                voting_period: 59,
                ..args.clone()
            },
            GovernanceError::InvalidVotingPeriod,
        ),
    ];
    for (bad, error) in cases {
        let result = env.send(
            &[ix::governance::initialize(&admin.pubkey(), bad)],
            &[&admin],
        );
        expect_error(result, error);
    }
    env.ok(
        &[ix::governance::initialize(&admin.pubkey(), args)],
        &[&admin],
    );
}
