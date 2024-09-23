use anchor_lang::prelude::account as pda;
use anchor_lang::prelude::*;

#[pda]
pub struct Amendment {
    pub id: u32,
    pub card_id: u8,
    pub new_metadata: Pubkey,
    pub deadline_slot: u128,
}

impl Amendment {
    pub const INIT_SPACE: usize = 8 + 4 + 1 + 32 + 16;
}
