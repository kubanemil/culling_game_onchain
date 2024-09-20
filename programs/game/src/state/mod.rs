use anchor_lang::prelude::account as pda;
use anchor_lang::prelude::*;

#[pda]
pub struct Playerx {
    pub level: u8,
    pub owner: Pubkey,
    pub game_won: u32,
    pub game_lost: u32,
    pub bump: u8,
}

#[pda]
pub struct Game {
    pub id: u32,
    pub stake_amount: u64,
    pub players: [Pubkey; 2],
    pub bump: u8,
}

impl  Game {
    pub const INIT_SPACE: usize = 8 + 4 + 8 + (32*2) + 1;
} 