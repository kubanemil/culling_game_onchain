use anchor_lang::prelude::*;

#[constant]
pub const DUEL_SEED: &[u8] = b"duel";

/// Every payout must keep an emptied wallet rent-exempt, or settling could be blocked.
#[constant]
pub const MIN_STAKE: u64 = 1_000_000;

/// Seconds to reveal, counted from accept.
#[constant]
pub const REVEAL_WINDOW: i64 = 60 * 60;
