use anchor_lang::prelude::*;

#[constant]
pub const DUEL_SEED: &[u8] = b"duel";

/// A revealed card waits in this token account until its owner claims.
#[constant]
pub const ESCROW_SEED: &[u8] = b"escrow";

/// Keeps every payout above the rent minimum of an empty wallet.
#[constant]
pub const MIN_STAKE: u64 = 1_000_000;

/// Seconds to reveal, counted from accept.
#[constant]
pub const REVEAL_WINDOW: i64 = 60 * 60;
