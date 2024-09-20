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

    pub fn initialize(ctx: Context<CreateGame>) -> Result<()> {
        ctx.accounts.create_game()
    }
}
