use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::SaleError,
    state::{Config, Round},
};

#[derive(Accounts)]
pub struct WithdrawProceeds<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin @ SaleError::Unauthorized,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [ROUND_SEED, &round.id.to_le_bytes()],
        bump = round.bump,
    )]
    pub round: Account<'info, Round>,
}

pub fn handle_withdraw_proceeds(ctx: Context<WithdrawProceeds>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let round = &mut ctx.accounts.round;
    require!(now >= round.end_ts, SaleError::RoundNotEnded);
    let amount = round.proceeds;
    require!(amount > 0, SaleError::NoProceeds);

    round.proceeds = 0;
    round.sub_lamports(amount)?;
    ctx.accounts.admin.add_lamports(amount)?;
    Ok(())
}
