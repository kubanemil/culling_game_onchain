use anchor_lang::prelude::*;

#[event]
pub struct DuelSettled {
    pub duel: Pubkey,
    pub creator: Pubkey,
    pub opponent: Pubkey,
    /// `None` on a draw or when neither player revealed.
    pub winner: Option<Pubkey>,
    pub stake: u64,
}
