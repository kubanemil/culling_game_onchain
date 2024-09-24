use anchor_lang::prelude::account as pda;
use anchor_lang::prelude::*;

#[pda]
pub struct Amendment {
    pub creator: Pubkey,
    pub card_id: u8,
    pub new_metadata_uri: String,
    pub deadline_slot: u64,
    pub pros: u64,
    pub cons: u64,
}

impl Amendment {
    pub const INIT_SPACE: usize = 8 + 32 + 1 + 32 + (3 * 8);
}

#[pda]
pub struct Vote {
    pub voter: Pubkey,
    pub amendment: Pubkey,
    pub accept: bool,
    pub tokens: u64,
}

impl Vote {
    pub const INIT_SPACE: usize = 8 + 32 + 32 + 1 + 8;
}
