use anchor_lang::prelude::*;
use asset::{Card, CardField, CARD_SEED};

use crate::{
    constants::*,
    error::GovernanceError,
    state::{Config, Proposal, ProposalStatus},
};

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(mut, seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [CARD_SEED, card.mint.as_ref()],
        bump = card.bump,
        seeds::program = asset::ID,
    )]
    pub card: Account<'info, Card>,

    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [PROPOSAL_SEED, &config.proposal_count.to_le_bytes()],
        bump,
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_proposal(
    ctx: Context<CreateProposal>,
    field: CardField,
    value: String,
) -> Result<()> {
    require!(field.accepts(&value), GovernanceError::InvalidValue);

    let now = Clock::get()?.unix_timestamp;
    let config = &mut ctx.accounts.config;
    let end_ts = now
        .checked_add(config.voting_period)
        .ok_or(GovernanceError::MathOverflow)?;

    ctx.accounts.proposal.set_inner(Proposal {
        id: config.proposal_count,
        proposer: ctx.accounts.proposer.key(),
        card: ctx.accounts.card.key(),
        card_mint: ctx.accounts.card.mint,
        field,
        value,
        start_ts: now,
        end_ts,
        yes: 0,
        no: 0,
        status: ProposalStatus::Voting,
        bump: ctx.bumps.proposal,
    });

    config.proposal_count = config
        .proposal_count
        .checked_add(1)
        .ok_or(GovernanceError::MathOverflow)?;
    Ok(())
}
