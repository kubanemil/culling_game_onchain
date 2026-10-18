use anchor_lang::{derive_program_address, prelude::*};

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const CARD_MINT_SEED: &[u8] = b"card_mint";

#[constant]
pub const CARD_SEED: &[u8] = b"card";

/// Must match `governance::ID`; an integration test keeps them in sync.
pub const GOVERNANCE_PROGRAM_ID: Pubkey = pubkey!("Bxd1GMH7cBj7XfsN5cXzK5idHGrpSXG5aHakgMCqYvdt");

pub const GOVERNANCE_AUTHORITY_SEED: &[u8] = b"authority";

/// The only signer allowed to change card metadata.
pub const GOVERNANCE_AUTHORITY: Pubkey = Pubkey::new_from_array(
    derive_program_address(
        &[GOVERNANCE_AUTHORITY_SEED],
        &GOVERNANCE_PROGRAM_ID.to_bytes(),
    )
    .0,
);

pub const MAX_STAT: u16 = 100;

pub const MAX_NAME_LEN: usize = 32;
pub const MAX_SYMBOL_LEN: usize = 10;
pub const MAX_URI_LEN: usize = 200;
