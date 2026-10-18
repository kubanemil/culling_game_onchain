pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use asset::CardField;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Bxd1GMH7cBj7XfsN5cXzK5idHGrpSXG5aHakgMCqYvdt");

#[program]
pub mod governance {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        instructions::initialize::handle_initialize(ctx, args)
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        field: CardField,
        value: String,
    ) -> Result<()> {
        instructions::create_proposal::handle_create_proposal(ctx, field, value)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handle_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::handle_withdraw(ctx, amount)
    }

    pub fn vote(ctx: Context<CastVote>, approve: bool) -> Result<()> {
        instructions::vote::handle_vote(ctx, approve)
    }

    pub fn close_vote(ctx: Context<CloseVote>) -> Result<()> {
        instructions::close_vote::handle_close_vote(ctx)
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        instructions::execute_proposal::handle_execute_proposal(ctx)
    }
}
