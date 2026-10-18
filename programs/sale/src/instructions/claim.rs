use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, Token2022, TokenAccount},
};

use crate::{
    constants::*,
    error::SaleError,
    events::Claimed,
    math::vested_amount,
    state::{Allocation, Config, Round},
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = mint)]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(seeds = [ROUND_SEED, &round.id.to_le_bytes()], bump = round.bump)]
    pub round: Account<'info, Round>,

    #[account(
        mut,
        seeds = [ALLOCATION_SEED, round.key().as_ref(), buyer.key().as_ref()],
        bump = allocation.bump,
        has_one = buyer,
        has_one = round,
    )]
    pub allocation: Account<'info, Allocation>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_tokens: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_claim(ctx: Context<Claim>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let accounts = ctx.accounts;
    let round = &accounts.round;
    let allocation = &accounts.allocation;

    let vested = vested_amount(
        allocation.purchased,
        round.end_ts,
        round.cliff,
        round.vesting,
        now,
    )?;
    let amount = vested
        .checked_sub(allocation.claimed)
        .ok_or(SaleError::MathOverflow)?;
    require!(amount > 0, SaleError::NothingToClaim);

    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[accounts.config.bump]]];
    mint_to(
        CpiContext::new_with_signer(
            accounts.token_program.key(),
            MintTo {
                mint: accounts.mint.to_account_info(),
                to: accounts.buyer_tokens.to_account_info(),
                authority: accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    emit!(Claimed {
        round: round.key(),
        buyer: accounts.buyer.key(),
        amount,
    });

    if vested == allocation.purchased {
        // Safe to close: claiming starts at round end, after which nobody can buy.
        accounts
            .allocation
            .close(accounts.buyer.to_account_info())?;
    } else {
        accounts.allocation.claimed = vested;
    }
    Ok(())
}
