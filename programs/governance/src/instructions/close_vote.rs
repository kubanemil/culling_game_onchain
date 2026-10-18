use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::GovernanceError,
    state::{Proposal, VoteRecord},
};

#[derive(Accounts)]
pub struct CloseVote<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [PROPOSAL_SEED, &proposal.id.to_le_bytes()],
        bump = proposal.bump,
    )]
    pub proposal: Account<'info, Proposal>,

    #[account(
        mut,
        close = owner,
        seeds = [VOTE_SEED, proposal.key().as_ref(), owner.key().as_ref()],
        bump = vote.bump,
    )]
    pub vote: Account<'info, VoteRecord>,
}

/// Voting needs `now < end_ts`, so a closed record cannot be recreated to vote again.
pub fn handle_close_vote(ctx: Context<CloseVote>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= ctx.accounts.proposal.end_ts,
        GovernanceError::VotingNotEnded
    );
    Ok(())
}
