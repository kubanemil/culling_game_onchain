pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("9iEyttWJGvGBApj51JLj8dJP9S3ZWdQB2fv3h7RCfLCB");

#[program]
pub mod governance {
    use super::*;

    pub fn vote(ctx: Context<Vote>) -> Result<()> {
        ctx.accounts.vote()
    }

    pub fn create_amendment(ctx: Context<CreateAmendement>) -> Result<()> {
        ctx.accounts.create()
    }

    pub fn resolve_amendment(ctx: Context<ResolveAmendment>) -> Result<()> {
        ctx.accounts.resolve()
    }
}
