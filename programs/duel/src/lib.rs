pub mod constants;
pub mod error;
pub mod events;
pub mod fight;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HcyprDcax4Ffz6h9s1sA8X4Zh6yWwZuipw2qFvxG6CkE");

#[program]
pub mod duel {
    use super::*;

    pub fn create_duel(
        ctx: Context<CreateDuel>,
        nonce: u64,
        stake: u64,
        commitment: [u8; 32],
        opponent: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_duel::handle_create_duel(ctx, nonce, stake, commitment, opponent)
    }

    pub fn accept_duel(ctx: Context<AcceptDuel>, commitment: [u8; 32]) -> Result<()> {
        instructions::accept_duel::handle_accept_duel(ctx, commitment)
    }

    pub fn cancel_duel(ctx: Context<CancelDuel>) -> Result<()> {
        instructions::cancel_duel::handle_cancel_duel(ctx)
    }

    pub fn reveal(ctx: Context<Reveal>, secret: [u8; 32]) -> Result<()> {
        instructions::reveal::handle_reveal(ctx, secret)
    }

    pub fn settle(ctx: Context<Settle>) -> Result<()> {
        instructions::settle::handle_settle(ctx)
    }
}
