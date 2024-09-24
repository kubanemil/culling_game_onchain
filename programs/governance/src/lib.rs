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

    pub fn vote(ctx: Context<VoteFor>, accept: bool, token_amount: u64) -> Result<()> {
        ctx.accounts.vote_for(accept, token_amount)
    }

    pub fn create_amendment(
        ctx: Context<CreateAmendement>,
        card_id: u8,
        new_metadata_uri: String,
        deadline_slot: u64,
    ) -> Result<()> {
        ctx.accounts
            .create(card_id, new_metadata_uri, deadline_slot)
    }

    pub fn resolve_amendment(ctx: Context<ResolveAmendment>) -> Result<()> {
        ctx.accounts.resolve()
    }
}
