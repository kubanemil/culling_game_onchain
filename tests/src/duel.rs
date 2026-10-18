use ::duel::{
    commitment,
    error::DuelError,
    fight::{fight, Outcome},
    randomness, Duel, DuelStatus, MIN_STAKE, REVEAL_WINDOW,
};
use anchor_lang::{
    error::ErrorCode,
    prelude::Pubkey,
    solana_program::{system_instruction, system_program},
    Space,
};
use anchor_spl::token_2022::spl_token_2022;
use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::{
    harness::{ata, create_ata_ix, expect_error, expect_log, TxResult},
    ix,
    world::{World, BRUTE, GUARD, TOKEN},
};

const STAKE: u64 = 1_000_000_000;
const FEE: u64 = 5_000;
const NONCE: u64 = 7;

/// Three funded players: creator holds Brute, opponent holds Guard, stranger holds Glass.
struct Table {
    world: World,
    creator: Keypair,
    opponent: Keypair,
    stranger: Keypair,
    duel: Pubkey,
    creator_secret: [u8; 32],
    opponent_secret: [u8; 32],
}

impl Table {
    fn new() -> Self {
        let mut world = World::new();
        let mut players = world.funded_players(3, 30 * TOKEN);
        let stranger = players.pop().unwrap();
        let opponent = players.pop().unwrap();
        let creator = players.pop().unwrap();
        world.buy_card(&creator, 0);
        world.buy_card(&opponent, 1);
        world.buy_card(&stranger, 2);
        let duel = ix::duel::address(&creator.pubkey(), NONCE);
        Self {
            world,
            creator,
            opponent,
            stranger,
            duel,
            creator_secret: [1; 32],
            opponent_secret: [2; 32],
        }
    }

    fn card(&self, index: usize) -> Pubkey {
        self.world.cards[index]
    }

    fn commit(&self, player: &Keypair, card: usize, secret: [u8; 32]) -> [u8; 32] {
        commitment(&self.duel, &player.pubkey(), &self.card(card), &secret)
    }

    fn create(&mut self, opponent: Option<Pubkey>) -> TxResult {
        let commitment = self.commit(&self.creator, 0, self.creator_secret);
        let ix = ix::duel::create(&self.creator.pubkey(), NONCE, STAKE, commitment, opponent);
        self.world.env.send(&[ix], &[&self.creator])
    }

    fn accept_as(&mut self, player: &Keypair, card: usize) -> TxResult {
        let commitment = self.commit(player, card, self.opponent_secret);
        let ix = ix::duel::accept(&player.pubkey(), &self.duel, commitment);
        self.world.env.send(&[ix], &[player])
    }

    fn accept(&mut self) -> TxResult {
        let opponent = self.opponent.insecure_clone();
        self.accept_as(&opponent, 1)
    }

    fn reveal_as(&mut self, player: &Keypair, card: usize, secret: [u8; 32]) -> TxResult {
        let ix = ix::duel::reveal(&player.pubkey(), &self.duel, &self.card(card), secret);
        self.world.env.send(&[ix], &[player])
    }

    fn reveal_creator(&mut self) -> TxResult {
        let creator = self.creator.insecure_clone();
        self.reveal_as(&creator, 0, self.creator_secret)
    }

    fn reveal_opponent(&mut self) -> TxResult {
        let opponent = self.opponent.insecure_clone();
        self.reveal_as(&opponent, 1, self.opponent_secret)
    }

    fn claim_as(&mut self, player: &Keypair, revealed: Option<usize>) -> TxResult {
        let card = revealed.map(|index| self.card(index));
        let ix = ix::duel::claim(&player.pubkey(), &self.duel, card.as_ref());
        self.world.env.send(&[ix], &[player])
    }

    /// Claims and returns how many lamports the player gained, fee included.
    fn claim_gain(&mut self, player: &Keypair, revealed: Option<usize>) -> i128 {
        let before = self.world.env.lamports(&player.pubkey()) as i128;
        self.claim_as(player, revealed).unwrap();
        self.world.env.lamports(&player.pubkey()) as i128 - before
    }

    fn state(&self) -> Duel {
        self.world.env.account(&self.duel)
    }

    fn escrow_rent(&self) -> u64 {
        self.world
            .env
            .lamports(&ix::duel::escrow(&self.duel, &self.creator.pubkey()))
    }

    fn duel_rent(&self) -> u64 {
        self.world
            .env
            .svm
            .minimum_balance_for_rent_exemption(8 + Duel::INIT_SPACE)
    }

    fn warp_past_deadline(&mut self) {
        let deadline = self.state().reveal_deadline;
        self.world.env.warp_to(deadline + 1);
    }
}

fn payouts(outcome: Outcome) -> (i128, i128) {
    let stake = STAKE as i128;
    match outcome {
        Outcome::CreatorWins => (2 * stake, 0),
        Outcome::OpponentWins => (0, 2 * stake),
        Outcome::Draw => (stake, stake),
    }
}

#[test]
fn a_full_duel_pays_each_side_its_share_and_returns_the_cards() {
    let mut t = Table::new();
    t.create(None).unwrap();
    t.accept().unwrap();
    let duel = t.state();
    assert_eq!(duel.status, DuelStatus::Accepted);
    assert_eq!(duel.reveal_deadline, t.world.env.now() + REVEAL_WINDOW);

    t.reveal_creator().unwrap();
    t.reveal_opponent().unwrap();
    assert_eq!(
        t.world
            .env
            .token_balance(&ata(&t.creator.pubkey(), &t.card(0))),
        0
    );

    let seed = randomness(&t.duel, &t.creator_secret, &t.opponent_secret);
    let (creator_share, opponent_share) = payouts(fight(&BRUTE, &GUARD, &seed));
    let escrow_rent = t.escrow_rent() as i128;
    let duel_rent = t.duel_rent() as i128;
    let opponent = t.opponent.insecure_clone();
    let creator = t.creator.insecure_clone();

    assert_eq!(
        t.claim_gain(&opponent, Some(1)),
        opponent_share + escrow_rent - FEE as i128
    );
    assert!(t.world.env.exists(&t.duel));
    assert_eq!(
        t.claim_gain(&creator, Some(0)),
        creator_share + escrow_rent + duel_rent - FEE as i128
    );
    assert!(!t.world.env.exists(&t.duel));
    assert_eq!(
        t.world
            .env
            .token_balance(&ata(&creator.pubkey(), &t.card(0))),
        1
    );
    assert_eq!(
        t.world
            .env
            .token_balance(&ata(&opponent.pubkey(), &t.card(1))),
        1
    );
}

#[test]
fn a_mirror_match_draw_refunds_both_stakes() {
    let mut t = Table::new();
    let creator = t.creator.insecure_clone();
    let opponent = t.opponent.insecure_clone();
    t.world.buy_card(&opponent, 0);

    let secret = (0u32..)
        .map(|i| {
            let mut s = [9; 32];
            s[..4].copy_from_slice(&i.to_le_bytes());
            s
        })
        .find(|s| {
            fight(&BRUTE, &BRUTE, &randomness(&t.duel, &t.creator_secret, s)) == Outcome::Draw
        })
        .unwrap();
    t.opponent_secret = secret;

    t.create(None).unwrap();
    t.accept_as(&opponent, 0).unwrap();
    t.reveal_as(&creator, 0, t.creator_secret).unwrap();
    t.reveal_as(&opponent, 0, secret).unwrap();

    let escrow_rent = t.escrow_rent() as i128;
    let duel_rent = t.duel_rent() as i128;
    assert_eq!(
        t.claim_gain(&opponent, Some(0)),
        STAKE as i128 + escrow_rent - FEE as i128
    );
    assert_eq!(
        t.claim_gain(&creator, Some(0)),
        STAKE as i128 + escrow_rent + duel_rent - FEE as i128
    );
}

#[test]
fn attack_accept_cannot_be_called_twice() {
    let mut t = Table::new();
    t.create(None).unwrap();
    t.accept().unwrap();

    let stranger = t.stranger.insecure_clone();
    expect_error(t.accept_as(&stranger, 2), DuelError::NotOpen);
    expect_error(t.accept(), DuelError::NotOpen);
    let rent = t.duel_rent();
    assert_eq!(t.world.env.lamports(&t.duel), 2 * STAKE + rent);
}

#[test]
fn attack_a_dust_stake_is_rejected() {
    let mut t = Table::new();
    let commitment = t.commit(&t.creator, 0, t.creator_secret);
    let ix = ix::duel::create(&t.creator.pubkey(), NONCE, MIN_STAKE - 1, commitment, None);
    expect_error(
        t.world.env.send(&[ix], &[&t.creator]),
        DuelError::StakeTooSmall,
    );
}

#[test]
fn attack_a_player_cannot_duel_themselves() {
    let mut t = Table::new();
    let creator = t.creator.insecure_clone();
    expect_error(t.create(Some(creator.pubkey())), DuelError::SelfPlay);

    t.create(None).unwrap();
    expect_error(t.accept_as(&creator, 0), DuelError::SelfPlay);
}

#[test]
fn attack_only_the_invited_opponent_can_accept() {
    let mut t = Table::new();
    t.create(Some(t.opponent.pubkey())).unwrap();
    let stranger = t.stranger.insecure_clone();
    expect_error(t.accept_as(&stranger, 2), DuelError::WrongOpponent);
    t.accept().unwrap();
}

#[test]
fn the_creator_can_cancel_before_accept_and_gets_everything_back() {
    let mut t = Table::new();
    let before = t.world.env.lamports(&t.creator.pubkey());
    t.create(None).unwrap();
    t.world.env.ok(
        &[ix::duel::cancel(&t.creator.pubkey(), &t.duel)],
        &[&t.creator],
    );
    assert!(!t.world.env.exists(&t.duel));
    assert_eq!(t.world.env.lamports(&t.creator.pubkey()), before - 2 * FEE);
}

#[test]
fn attack_nobody_else_can_cancel_and_nobody_can_cancel_after_accept() {
    let mut t = Table::new();
    t.create(None).unwrap();
    let stranger = t.stranger.insecure_clone();
    let result = t.world.env.send(
        &[ix::duel::cancel(&stranger.pubkey(), &t.duel)],
        &[&stranger],
    );
    expect_error(result, ErrorCode::ConstraintSeeds);

    t.accept().unwrap();
    let result = t.world.env.send(
        &[ix::duel::cancel(&t.creator.pubkey(), &t.duel)],
        &[&t.creator],
    );
    expect_error(result, DuelError::NotOpen);
}

#[test]
fn attack_a_wrong_secret_or_card_does_not_match_the_commitment() {
    let mut t = Table::new();
    let creator = t.creator.insecure_clone();
    t.world.buy_card(&creator, 2);
    t.create(None).unwrap();
    t.accept().unwrap();

    expect_error(
        t.reveal_as(&creator, 0, [3; 32]),
        DuelError::CommitmentMismatch,
    );
    expect_error(
        t.reveal_as(&creator, 2, t.creator_secret),
        DuelError::CommitmentMismatch,
    );
    t.reveal_creator().unwrap();
    expect_log(t.reveal_creator(), "already in use");
}

#[test]
fn attack_a_player_cannot_reveal_a_card_they_no_longer_hold() {
    let mut t = Table::new();
    t.create(None).unwrap();
    t.accept().unwrap();
    let card = t.card(0);
    let stranger = t.stranger.pubkey();
    let creator = t.creator.insecure_clone();
    t.world
        .env
        .transfer_tokens(&creator, &card, &stranger, 1, 0);

    expect_error(t.reveal_creator(), DuelError::CardNotOwned);
}

#[test]
fn attack_one_card_cannot_back_two_reveals_at_once() {
    let mut t = Table::new();
    let creator = t.creator.insecure_clone();
    let stranger = t.stranger.insecure_clone();
    let first = t.duel;
    let second = ix::duel::address(&creator.pubkey(), NONCE + 1);

    t.create(None).unwrap();
    t.accept().unwrap();
    t.duel = second;
    let commitment = t.commit(&creator, 0, t.creator_secret);
    t.world.env.ok(
        &[ix::duel::create(
            &creator.pubkey(),
            NONCE + 1,
            STAKE,
            commitment,
            None,
        )],
        &[&creator],
    );
    t.accept_as(&stranger, 2).unwrap();

    t.duel = first;
    t.reveal_creator().unwrap();
    assert_eq!(
        t.world
            .env
            .token_balance(&ix::duel::escrow(&first, &creator.pubkey())),
        1
    );

    t.duel = second;
    expect_error(t.reveal_creator(), DuelError::CardNotOwned);
}

#[test]
fn attack_a_card_borrowed_for_one_transaction_cannot_be_handed_back_in_it() {
    let mut t = Table::new();
    let opponent = t.opponent.insecure_clone();
    let lender = t.stranger.insecure_clone();
    let glass = t.card(2);
    t.create(None).unwrap();
    t.accept_as(&opponent, 2).unwrap();

    let transfer = |from: &Pubkey, to: &Pubkey| {
        spl_token_2022::instruction::transfer_checked(
            &spl_token_2022::ID,
            &ata(from, &glass),
            &glass,
            &ata(to, &glass),
            from,
            &[],
            1,
            0,
        )
        .unwrap()
    };
    let ixs = [
        create_ata_ix(&opponent.pubkey(), &opponent.pubkey(), &glass),
        transfer(&lender.pubkey(), &opponent.pubkey()),
        ix::duel::reveal(&opponent.pubkey(), &t.duel, &glass, t.opponent_secret),
        transfer(&opponent.pubkey(), &lender.pubkey()),
    ];
    expect_log(
        t.world.env.send(&ixs, &[&opponent, &lender]),
        "insufficient funds",
    );
    assert_eq!(t.world.env.token_balance(&ata(&lender.pubkey(), &glass)), 1);
}

#[test]
fn attack_a_stranger_cannot_reveal_in_someone_elses_duel() {
    let mut t = Table::new();
    t.create(None).unwrap();
    t.accept().unwrap();
    let stranger = t.stranger.insecure_clone();
    expect_error(t.reveal_as(&stranger, 2, [5; 32]), DuelError::NotAPlayer);
}

#[test]
fn attack_a_non_revealing_player_forfeits_after_timeout() {
    let mut t = Table::new();
    let creator = t.creator.insecure_clone();
    let opponent = t.opponent.insecure_clone();
    t.create(None).unwrap();
    t.accept().unwrap();
    t.reveal_creator().unwrap();

    expect_error(t.claim_as(&creator, Some(0)), DuelError::RevealWindowOpen);

    t.warp_past_deadline();
    expect_error(t.reveal_opponent(), DuelError::RevealWindowClosed);

    let escrow_rent = t.escrow_rent() as i128;
    let duel_rent = t.duel_rent() as i128;
    assert_eq!(
        t.claim_gain(&creator, Some(0)),
        2 * STAKE as i128 + escrow_rent + duel_rent - FEE as i128
    );
    assert!(!t.world.env.exists(&t.duel));
    expect_error(
        t.claim_as(&opponent, None),
        ErrorCode::AccountNotInitialized,
    );
}

#[test]
fn if_nobody_reveals_both_stakes_are_refunded_after_timeout() {
    let mut t = Table::new();
    let creator = t.creator.insecure_clone();
    let opponent = t.opponent.insecure_clone();
    t.create(None).unwrap();
    t.accept().unwrap();
    t.warp_past_deadline();

    let duel_rent = t.duel_rent() as i128;
    assert_eq!(t.claim_gain(&creator, None), STAKE as i128 - FEE as i128);
    assert!(t.world.env.exists(&t.duel));
    assert_eq!(t.claim_gain(&opponent, None), STAKE as i128 - FEE as i128);
    assert_eq!(t.claim_gain(&creator, None), duel_rent - FEE as i128);
    assert!(!t.world.env.exists(&t.duel));
}

#[test]
fn attack_a_loser_cannot_lock_the_pot_by_reassigning_their_wallet() {
    let mut t = Table::new();
    let creator = t.creator.insecure_clone();
    let opponent = t.opponent.insecure_clone();
    t.opponent_secret = (0u32..)
        .map(|i| {
            let mut s = [4; 32];
            s[..4].copy_from_slice(&i.to_le_bytes());
            s
        })
        .find(|s| {
            let seed = randomness(&t.duel, &t.creator_secret, s);
            fight(&BRUTE, &GUARD, &seed) == Outcome::OpponentWins
        })
        .unwrap();

    t.create(None).unwrap();
    t.accept().unwrap();
    t.reveal_creator().unwrap();
    t.reveal_opponent().unwrap();

    // The losing creator hands their wallet to another program, which used to make any
    // payment or close into it fail.
    t.world.env.ok(
        &[system_instruction::assign(&creator.pubkey(), &::asset::ID)],
        &[&creator],
    );
    assert_ne!(
        t.world
            .env
            .svm
            .get_account(&creator.pubkey())
            .unwrap()
            .owner,
        system_program::ID
    );

    let escrow_rent = t.escrow_rent() as i128;
    assert_eq!(
        t.claim_gain(&opponent, Some(1)),
        2 * STAKE as i128 + escrow_rent - FEE as i128
    );
}

#[test]
fn attack_strangers_cannot_claim_and_players_cannot_claim_twice() {
    let mut t = Table::new();
    let opponent = t.opponent.insecure_clone();
    let stranger = t.stranger.insecure_clone();
    t.create(None).unwrap();
    expect_error(t.claim_as(&opponent, None), DuelError::NotAccepted);

    t.accept().unwrap();
    t.warp_past_deadline();
    expect_error(t.claim_as(&stranger, None), DuelError::NotAPlayer);
    t.claim_as(&opponent, None).unwrap();
    expect_error(t.claim_as(&opponent, None), DuelError::AlreadyClaimed);
}
