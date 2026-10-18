use ::duel::{
    commitment,
    error::DuelError,
    fight::{fight, Outcome},
    randomness, Duel, DuelStatus, MIN_STAKE, REVEAL_WINDOW,
};
use anchor_lang::{error::ErrorCode, prelude::Pubkey, Space};
use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::{
    harness::{expect_error, TxResult},
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
    cranker: Keypair,
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
        let cranker = world.env.user();
        let duel = ix::duel::address(&creator.pubkey(), NONCE);
        Self {
            world,
            creator,
            opponent,
            stranger,
            cranker,
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

    fn settle_with(&mut self, opponent: &Pubkey) -> TxResult {
        let ix = ix::duel::settle(&self.duel, &self.creator.pubkey(), opponent);
        self.world.env.send(&[ix], &[&self.cranker])
    }

    fn settle(&mut self) -> TxResult {
        let opponent = self.opponent.pubkey();
        self.settle_with(&opponent)
    }

    fn state(&self) -> Duel {
        self.world.env.account(&self.duel)
    }

    fn balances(&self) -> (u64, u64) {
        (
            self.world.env.lamports(&self.creator.pubkey()),
            self.world.env.lamports(&self.opponent.pubkey()),
        )
    }

    fn warp_past_deadline(&mut self) {
        let deadline = self.state().reveal_deadline;
        self.world.env.warp_to(deadline + 1);
    }
}

#[test]
fn a_full_duel_pays_the_winner_from_the_pda_and_closes_it() {
    let mut t = Table::new();
    t.create(None).unwrap();
    t.accept().unwrap();
    let duel = t.state();
    assert_eq!(duel.status, DuelStatus::Accepted);
    assert_eq!(duel.opponent, Some(t.opponent.pubkey()));
    assert_eq!(duel.reveal_deadline, t.world.env.now() + REVEAL_WINDOW);

    t.reveal_creator().unwrap();
    t.reveal_opponent().unwrap();

    let rent = t.world.env.lamports(&t.duel) - 2 * STAKE;
    let (creator_before, opponent_before) = t.balances();
    let seed = randomness(&t.duel, &t.creator_secret, &t.opponent_secret);
    let outcome = fight(&BRUTE, &GUARD, &seed);
    t.settle().unwrap();

    let (creator_after, opponent_after) = t.balances();
    let (creator_gain, opponent_gain) = match outcome {
        Outcome::CreatorWins => (2 * STAKE + rent, 0),
        Outcome::OpponentWins => (rent, 2 * STAKE),
        Outcome::Draw => (STAKE + rent, STAKE),
    };
    assert_eq!(creator_after - creator_before, creator_gain);
    assert_eq!(opponent_after - opponent_before, opponent_gain);
    assert!(!t.world.env.exists(&t.duel));
}

#[test]
fn a_draw_refunds_both_stakes() {
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

    let rent = t.world.env.lamports(&t.duel) - 2 * STAKE;
    let (creator_before, opponent_before) = t.balances();
    t.settle().unwrap();
    let (creator_after, opponent_after) = t.balances();
    assert_eq!(creator_after - creator_before, STAKE + rent);
    assert_eq!(opponent_after - opponent_before, STAKE);
}

#[test]
fn attack_accept_cannot_be_called_twice() {
    let mut t = Table::new();
    t.create(None).unwrap();
    t.accept().unwrap();

    let stranger = t.stranger.insecure_clone();
    expect_error(t.accept_as(&stranger, 2), DuelError::NotOpen);
    expect_error(t.accept(), DuelError::NotOpen);
    let rent = t
        .world
        .env
        .svm
        .minimum_balance_for_rent_exemption(8 + Duel::INIT_SPACE);
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
    expect_error(t.reveal_creator(), DuelError::AlreadyRevealed);
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
    t.create(None).unwrap();
    t.accept().unwrap();
    t.reveal_creator().unwrap();

    expect_error(t.settle(), DuelError::RevealWindowOpen);

    t.warp_past_deadline();
    expect_error(t.reveal_opponent(), DuelError::RevealWindowClosed);

    let rent = t.world.env.lamports(&t.duel) - 2 * STAKE;
    let (creator_before, opponent_before) = t.balances();
    t.settle().unwrap();
    let (creator_after, opponent_after) = t.balances();
    assert_eq!(creator_after - creator_before, 2 * STAKE + rent);
    assert_eq!(opponent_after, opponent_before);
}

#[test]
fn if_nobody_reveals_both_stakes_are_refunded_after_timeout() {
    let mut t = Table::new();
    t.create(None).unwrap();
    t.accept().unwrap();
    t.warp_past_deadline();

    let rent = t.world.env.lamports(&t.duel) - 2 * STAKE;
    let (creator_before, opponent_before) = t.balances();
    t.settle().unwrap();
    let (creator_after, opponent_after) = t.balances();
    assert_eq!(creator_after - creator_before, STAKE + rent);
    assert_eq!(opponent_after - opponent_before, STAKE);
}

#[test]
fn attack_settle_rejects_a_swapped_payout_account() {
    let mut t = Table::new();
    t.create(None).unwrap();
    expect_error(t.settle(), DuelError::NotAccepted);

    t.accept().unwrap();
    t.reveal_creator().unwrap();
    t.reveal_opponent().unwrap();
    let stranger = t.stranger.pubkey();
    expect_error(t.settle_with(&stranger), DuelError::NotAPlayer);
    t.settle().unwrap();
}
