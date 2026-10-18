pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod math;
pub mod merkle;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("64B6Rmfe4Jg5R4xdPCG3mHpaSihYJLyswC7dPutNQyjo");

#[program]
pub mod sale {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        instructions::initialize::handle_initialize(ctx, args)
    }

    pub fn propose_admin(ctx: Context<ProposeAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::transfer_admin::handle_propose_admin(ctx, new_admin)
    }

    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        instructions::transfer_admin::handle_accept_admin(ctx)
    }

    pub fn create_round(ctx: Context<CreateRound>, args: RoundArgs) -> Result<()> {
        instructions::create_round::handle_create_round(ctx, args)
    }

    pub fn set_paused(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
        instructions::set_paused::handle_set_paused(ctx, paused)
    }

    pub fn buy(
        ctx: Context<Buy>,
        amount: u64,
        max_amount: u64,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::buy::handle_buy(ctx, amount, max_amount, proof)
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        instructions::claim::handle_claim(ctx)
    }

    pub fn withdraw_proceeds(ctx: Context<WithdrawProceeds>) -> Result<()> {
        instructions::withdraw_proceeds::handle_withdraw_proceeds(ctx)
    }
}
