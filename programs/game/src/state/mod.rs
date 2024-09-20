use anchor_lang::prelude::*;

#[account]
pub struct Playerx {
    level: u8,
    owner: Pubkey,
    game_won: u32,
    game_lost: u32,
    bump: u8,
}

#[account]
pub struct Game {
    id: u32,
    stake_amount: u64,
    players: [Pubkey; 2],
    bump: u8,
}
