use anchor_lang::{derive_program_address, prelude::*};

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const PROPOSAL_SEED: &[u8] = b"proposal";

#[constant]
pub const VOTER_SEED: &[u8] = b"voter";

#[constant]
pub const VOTE_SEED: &[u8] = b"vote";

#[constant]
pub const FIELD_SEED: &[u8] = b"field";

pub const AUTHORITY_SEED: &[u8] = asset::GOVERNANCE_AUTHORITY_SEED;

const AUTHORITY_PDA: ([u8; 32], u8) =
    derive_program_address(&[AUTHORITY_SEED], &crate::ID_CONST.to_bytes());
pub const AUTHORITY: Pubkey = Pubkey::new_from_array(AUTHORITY_PDA.0);
pub const AUTHORITY_BUMP: u8 = AUTHORITY_PDA.1;

/// Seconds after voting ends during which a passed proposal can run. After that it expires.
#[constant]
pub const EXECUTION_WINDOW: i64 = 7 * 24 * 60 * 60;

pub const MIN_VOTING_PERIOD: i64 = 60;
pub const MAX_VOTING_PERIOD: i64 = 30 * 24 * 60 * 60;

/// Basis points of the votes cast that must approve, strictly more than this.
pub const MIN_THRESHOLD_BPS: u16 = 5_000;
pub const MAX_THRESHOLD_BPS: u16 = 9_999;
