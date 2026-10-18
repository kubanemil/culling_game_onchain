use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub pending_admin: Option<Pubkey>,
    pub mint: Pubkey,
    pub decimals: u8,
    /// Sum of all round hard caps can never exceed this.
    pub supply_cap: u64,
    pub allocated: u64,
    pub round_count: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Round {
    pub id: u64,
    pub start_ts: i64,
    pub end_ts: i64,
    /// Lamports per whole token (10^decimals base units).
    pub price: u64,
    pub hard_cap: u64,
    pub sold: u64,
    /// Lamports held by this account for the admin.
    pub proceeds: u64,
    pub merkle_root: [u8; 32],
    /// Vesting starts at `end_ts`. Seconds until anything unlocks.
    pub cliff: i64,
    /// Seconds from `end_ts` until everything is unlocked.
    pub vesting: i64,
    pub paused: bool,
    pub bump: u8,
}

/// One per wallet per round: the allowlisted cap, what was bought and what was claimed.
#[account]
#[derive(InitSpace)]
pub struct Allocation {
    pub round: Pubkey,
    pub buyer: Pubkey,
    pub max_amount: u64,
    pub purchased: u64,
    pub claimed: u64,
    pub bump: u8,
}
