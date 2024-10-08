use anchor_lang::prelude::account as pda;
use anchor_lang::prelude::*;

#[pda]
pub struct Player {
    pub initiated: bool,
    pub owner: Pubkey,
    pub game_won: u32,
    pub game_lost: u32,
}

impl Player {
    pub const INIT_SPACE: usize = 8 + 32 + (4 * 2) + 1;
}

#[pda]
pub struct Game {
    pub id: u32,
    pub stake_amount: u64,
    pub creator: Pubkey,
    pub opponent: Pubkey,
    pub accepted: bool,
    pub bump: u8,
}

impl Game {
    pub const INIT_SPACE: usize = 8 + 4 + 8 + (32 * 2) + 1 + 1;
}

#[pda]
pub struct Config {
    pub auth: Pubkey,
    pub owner: Pubkey,
    pub bump: u8,
}

impl Config {
    pub const INIT_SPACE: usize = 8 + (32 * 2) + 1;
}
