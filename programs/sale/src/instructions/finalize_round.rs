use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::SaleError,
    state::{Config, Round},
};

#[derive(Accounts)]
pub struct FinalizeRound<'info> {
    #[account(mut, seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [ROUND_SEED, &round.id.to_le_bytes()],
        bump = round.bump,
    )]
    pub round: Account<'info, Round>,
}

/// Anyone can call this once the round ends. The unsold part of the hard cap goes back
/// to the supply cap, so later rounds can sell it.
pub fn handle_finalize_round(ctx: Context<FinalizeRound>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let round = &mut ctx.accounts.round;
    require!(now >= round.end_ts, SaleError::RoundNotEnded);
    require!(!round.finalized, SaleError::AlreadyFinalized);

    let unsold = round
        .hard_cap
        .checked_sub(round.sold)
        .ok_or(SaleError::MathOverflow)?;
    let config = &mut ctx.accounts.config;
    config.allocated = config
        .allocated
        .checked_sub(unsold)
        .ok_or(SaleError::MathOverflow)?;
    round.finalized = true;
    Ok(())
}
