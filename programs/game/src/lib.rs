pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("5pgzgEQZ18P4w3iTy5dBjv3H4My2GgCM4FxXnHdxSBb7");

#[program]
pub mod game {
    use super::*;

    pub fn create_game(ctx: Context<CreateGame>, game_id: u32, stake_amount: u64) -> Result<()> {
        ctx.accounts.create_game(game_id, stake_amount, ctx.bumps)
    }

    pub fn accept_game(ctx: Context<AcceptGame>, game_id: u32) -> Result<()> {
        ctx.accounts.accept_game(game_id)
    }
}
