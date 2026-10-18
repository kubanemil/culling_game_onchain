use anchor_lang::prelude::*;
use asset::Stats;
use solana_sha256_hasher::hashv;

use crate::{
    error::DuelError,
    fight::{fight, Outcome},
};

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DuelStatus {
    Open,
    Accepted,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Revealed {
    pub card: Pubkey,
    /// Copied at reveal so the outcome needs no card accounts.
    pub stats: Stats,
    pub secret: [u8; 32],
}

/// Holds both stakes until each player claims their share. Closed by the creator's claim.
#[account]
#[derive(InitSpace)]
pub struct Duel {
    pub creator: Pubkey,
    pub nonce: u64,
    pub opponent: Option<Pubkey>,
    pub stake: u64,
    pub status: DuelStatus,
    pub creator_commitment: [u8; 32],
    pub opponent_commitment: [u8; 32],
    pub reveal_deadline: i64,
    pub creator_reveal: Option<Revealed>,
    pub opponent_reveal: Option<Revealed>,
    pub creator_claimed: bool,
    pub opponent_claimed: bool,
    pub bump: u8,
}

impl Duel {
    /// Both revealed: the cards fight. Window closed: a missing reveal forfeits, and if
    /// neither revealed it is a draw.
    pub fn outcome(&self, duel: &Pubkey, now: i64) -> Result<Outcome> {
        match (&self.creator_reveal, &self.opponent_reveal) {
            (Some(creator), Some(opponent)) => Ok(fight(
                &creator.stats,
                &opponent.stats,
                &randomness(duel, &creator.secret, &opponent.secret),
            )),
            _ if now <= self.reveal_deadline => err!(DuelError::RevealWindowOpen),
            (Some(_), None) => Ok(Outcome::CreatorWins),
            (None, Some(_)) => Ok(Outcome::OpponentWins),
            (None, None) => Ok(Outcome::Draw),
        }
    }

    /// Lamports owed to (creator, opponent).
    pub fn payouts(&self, outcome: Outcome) -> Result<(u64, u64)> {
        let pot = self.stake.checked_mul(2).ok_or(DuelError::MathOverflow)?;
        Ok(match outcome {
            Outcome::CreatorWins => (pot, 0),
            Outcome::OpponentWins => (0, pot),
            Outcome::Draw => (self.stake, self.stake),
        })
    }
}

/// Bound to the duel and the player, so a commitment cannot be copied or replayed.
pub fn commitment(duel: &Pubkey, player: &Pubkey, card: &Pubkey, secret: &[u8; 32]) -> [u8; 32] {
    hashv(&[duel.as_ref(), player.as_ref(), card.as_ref(), secret]).to_bytes()
}

/// Neither player can predict this until both secrets are revealed.
pub fn randomness(
    duel: &Pubkey,
    creator_secret: &[u8; 32],
    opponent_secret: &[u8; 32],
) -> [u8; 32] {
    hashv(&[duel.as_ref(), creator_secret, opponent_secret]).to_bytes()
}
