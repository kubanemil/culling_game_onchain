use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::SaleError,
    state::{Config, Round},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RoundArgs {
    pub start_ts: i64,
    pub end_ts: i64,
    pub price: u64,
    pub hard_cap: u64,
    pub merkle_root: [u8; 32],
    pub cliff: i64,
    pub vesting: i64,
}

#[derive(Accounts)]
pub struct CreateRound<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin @ SaleError::Unauthorized,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = admin,
        space = 8 + Round::INIT_SPACE,
        seeds = [ROUND_SEED, &config.round_count.to_le_bytes()],
        bump,
    )]
    pub round: Account<'info, Round>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_round(ctx: Context<CreateRound>, args: RoundArgs) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        args.end_ts > args.start_ts && args.end_ts > now,
        SaleError::InvalidSchedule
    );
    require!(
        args.price > 0 && args.hard_cap > 0,
        SaleError::InvalidPricing
    );
    require!(
        0 <= args.cliff && args.cliff <= args.vesting && args.vesting <= MAX_VESTING_SECONDS,
        SaleError::InvalidVesting
    );

    let config = &mut ctx.accounts.config;
    let allocated = config
        .allocated
        .checked_add(args.hard_cap)
        .ok_or(SaleError::MathOverflow)?;
    require!(allocated <= config.supply_cap, SaleError::SupplyCapExceeded);

    ctx.accounts.round.set_inner(Round {
        id: config.round_count,
        start_ts: args.start_ts,
        end_ts: args.end_ts,
        price: args.price,
        hard_cap: args.hard_cap,
        sold: 0,
        proceeds: 0,
        merkle_root: args.merkle_root,
        cliff: args.cliff,
        vesting: args.vesting,
        paused: false,
        finalized: false,
        bump: ctx.bumps.round,
    });

    config.allocated = allocated;
    config.round_count = config
        .round_count
        .checked_add(1)
        .ok_or(SaleError::MathOverflow)?;
    Ok(())
}
