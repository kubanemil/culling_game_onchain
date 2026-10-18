use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};

use crate::{
    constants::*,
    error::GovernanceError,
    events::VoteCast,
    state::{Config, Proposal, ProposalStatus, VoteRecord, Voter},
};

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = mint)]
    pub config: Account<'info, Config>,

    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [PROPOSAL_SEED, &proposal.id.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        seeds = [VOTER_SEED, owner.key().as_ref()],
        bump = voter.bump,
        has_one = owner,
    )]
    pub voter: Account<'info, Voter>,

    /// Only the pinned mint counts, so a self-minted token has no weight here.
    #[account(
        associated_token::mint = mint,
        associated_token::authority = voter,
        associated_token::token_program = token_program,
    )]
    pub escrow: Box<InterfaceAccount<'info, TokenAccount>>,

    /// One per voter per proposal: a second vote fails because this already exists.
    #[account(
        init,
        payer = owner,
        space = 8 + VoteRecord::INIT_SPACE,
        seeds = [VOTE_SEED, proposal.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub vote: Account<'info, VoteRecord>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_vote(ctx: Context<CastVote>, approve: bool) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let proposal = &mut ctx.accounts.proposal;
    require!(
        proposal.status == ProposalStatus::Voting && now < proposal.end_ts,
        GovernanceError::VotingEnded
    );
    let weight = ctx.accounts.escrow.amount;
    require!(weight > 0, GovernanceError::NoVotingPower);

    let tally = if approve {
        &mut proposal.yes
    } else {
        &mut proposal.no
    };
    *tally = tally
        .checked_add(weight)
        .ok_or(GovernanceError::MathOverflow)?;

    let voter = &mut ctx.accounts.voter;
    voter.locked_until = voter.locked_until.max(proposal.end_ts);

    ctx.accounts.vote.set_inner(VoteRecord {
        proposal: proposal.key(),
        voter: ctx.accounts.owner.key(),
        approve,
        weight,
        bump: ctx.bumps.vote,
    });

    emit!(VoteCast {
        proposal: proposal.key(),
        voter: ctx.accounts.owner.key(),
        approve,
        weight,
    });
    Ok(())
}
