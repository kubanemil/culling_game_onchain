use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, Token2022, TokenAccount, TransferChecked,
};

use crate::{
    constants::*,
    error::GovernanceError,
    state::{Config, Voter},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub owner: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = mint)]
    pub config: Account<'info, Config>,

    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [VOTER_SEED, owner.key().as_ref()],
        bump = voter.bump,
        has_one = owner,
    )]
    pub voter: Account<'info, Voter>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = voter,
        associated_token::token_program = token_program,
    )]
    pub escrow: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = owner,
        token::token_program = token_program,
    )]
    pub owner_tokens: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handle_withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    require!(amount > 0, GovernanceError::ZeroAmount);
    let now = Clock::get()?.unix_timestamp;
    let voter = &ctx.accounts.voter;
    let locked = if now < voter.locked_until {
        voter.locked_amount
    } else {
        0
    };
    require!(
        amount <= ctx.accounts.escrow.amount.saturating_sub(locked),
        GovernanceError::TokensLocked
    );

    let owner = ctx.accounts.owner.key();
    let signer_seeds: &[&[&[u8]]] = &[&[VOTER_SEED, owner.as_ref(), &[ctx.accounts.voter.bump]]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.escrow.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.owner_tokens.to_account_info(),
                authority: ctx.accounts.voter.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
        ctx.accounts.mint.decimals,
    )
}
