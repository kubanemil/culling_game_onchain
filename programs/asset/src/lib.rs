pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BbkymsEZeYP5uGvYAkrcnBvCTyvXH6bCtmSm6Dd8pUSh");

#[program]
pub mod asset {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.init(ctx.bumps)
    }

    pub fn buy_token(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
        ctx.accounts.buy_token(amount)
    }
}
