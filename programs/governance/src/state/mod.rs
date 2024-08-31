use anchor_lang::prelude::*;


#[account]
pub struct Amendment {
    id: u64,
    card_id: u8,
    slot: u128,
    new_property: String,
}

impl Amendment {
    const INIT_SPACE: usize = 8 + 8 + 1 + 16 + 64;
}
