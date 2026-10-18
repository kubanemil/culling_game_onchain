use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::DuelError,
    state::{Duel, DuelStatus},
};

#[derive(Accounts)]
pub struct CancelDuel<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Closing returns the stake and the rent in one step.
    #[account(
        mut,
        close = creator,
        has_one = creator,
        seeds = [DUEL_SEED, creator.key().as_ref(), &duel.nonce.to_le_bytes()],
        bump = duel.bump,
        constraint = duel.status == DuelStatus::Open @ DuelError::NotOpen,
    )]
    pub duel: Account<'info, Duel>,
}

pub fn handle_cancel_duel(_ctx: Context<CancelDuel>) -> Result<()> {
    Ok(())
}
