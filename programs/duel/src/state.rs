use anchor_lang::prelude::*;
use asset::Stats;
use solana_sha256_hasher::hashv;

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DuelStatus {
    Open,
    Accepted,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Revealed {
    pub card: Pubkey,
    /// Snapshot at reveal time, so a later metadata vote cannot change a live duel.
    pub stats: Stats,
    pub secret: [u8; 32],
}

/// Holds both stakes until `settle` pays out and closes it.
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
    pub bump: u8,
}

/// What a player commits to: the card they will play and a random secret, bound to this
/// duel and this player so a commitment cannot be copied or replayed.
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
