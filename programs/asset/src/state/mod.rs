use anchor_lang::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Grade {
    Third,
    Second,
    First,
    Special,
}

#[account]
pub struct Card {
    pub hp: u8,
    pub grade: u8,
    pub metadata: String,
}

// impl Card {
//     const INIT_SPACE: usize = 8 + 1 + 1 + 64;
// }

#[account]
pub struct AuthVault {
    pub owner: Pubkey,
    pub bump: u8,
    pub mint_bump: u8,
}

impl AuthVault {
    pub const INIT_SPACE: usize = 8 + 32 + 1 + 1;
}
