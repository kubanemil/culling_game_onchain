use anchor_lang::{derive_program_address, prelude::*};

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const MINT_SEED: &[u8] = b"mint";

#[constant]
pub const ROUND_SEED: &[u8] = b"round";

#[constant]
pub const ALLOCATION_SEED: &[u8] = b"allocation";

/// The game token. Other programs pin it by address.
pub const TOKEN_MINT: Pubkey =
    Pubkey::new_from_array(derive_program_address(&[MINT_SEED], &crate::ID_CONST.to_bytes()).0);

pub const MAX_DECIMALS: u8 = 9;

/// Enough for an allowlist of 16M wallets.
pub const MAX_PROOF_LEN: usize = 24;

pub const MAX_VESTING_SECONDS: i64 = 10 * 365 * 24 * 60 * 60;
