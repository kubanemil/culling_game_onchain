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

    pub fn vote(
        ctx: Context<VoteFor>,
        amendment_id: u32,
        accept: bool,
        token_amount: u64,
    ) -> Result<()> {
        ctx.accounts.vote_for(amendment_id, accept, token_amount)
    }

    pub fn create_amendment(
        ctx: Context<CreateAmendement>,
        amendment_id: u32,
        card_id: u8,
        new_metadata: Pubkey,
        deadline_slot: u128,
    ) -> Result<()> {
        ctx.accounts
            .create(amendment_id, card_id, new_metadata, deadline_slot)
    }

    pub fn resolve_amendment(ctx: Context<ResolveAmendment>) -> Result<()> {
        ctx.accounts.resolve()
    }
}
