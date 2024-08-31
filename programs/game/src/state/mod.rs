use anchor_lang::prelude::*;


#[account]
#[derive(InitSpace)]
pub struct Game {
    id: u64,
    player1: Pubkey,
    player2: Pubkey,
    stake: Pubkey
}

#[account]
#[derive(InitSpace)]
pub struct Player {
    level: u8,
    address: Pubkey,
    game_won: u64,
    game_lose: u64
}