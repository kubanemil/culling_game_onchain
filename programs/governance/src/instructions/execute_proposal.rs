use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, Token2022};
use asset::{cpi::accounts::UpdateCardMetadata, program::Asset, Card};

use crate::{
    constants::*,
    error::GovernanceError,
    events::ProposalExecuted,
    state::{Config, Proposal, ProposalStatus},
};

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    /// Anyone can execute a passed proposal. Pays any extra rent for longer metadata.
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [PROPOSAL_SEED, &proposal.id.to_le_bytes()],
        bump = proposal.bump,
        has_one = card,
        has_one = card_mint,
    )]
    pub proposal: Account<'info, Proposal>,

    /// CHECK: PDA that signs the CPI. Verified by seeds; holds no data.
    #[account(seeds = [AUTHORITY_SEED], bump = AUTHORITY_BUMP)]
    pub authority: UncheckedAccount<'info>,

    #[account(
        seeds = [asset::CONFIG_SEED],
        bump = asset_config.bump,
        seeds::program = asset::ID,
    )]
    pub asset_config: Account<'info, asset::Config>,

    pub card: Account<'info, Card>,

    #[account(mut)]
    pub card_mint: Box<InterfaceAccount<'info, Mint>>,

    pub asset_program: Program<'info, Asset>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let proposal = &ctx.accounts.proposal;
    require!(
        proposal.status == ProposalStatus::Voting,
        GovernanceError::AlreadyExecuted
    );
    require!(now >= proposal.end_ts, GovernanceError::VotingNotEnded);
    proposal.check_passed(&ctx.accounts.config)?;

    let signer_seeds: &[&[&[u8]]] = &[&[AUTHORITY_SEED, &[AUTHORITY_BUMP]]];
    asset::cpi::update_card_metadata(
        CpiContext::new_with_signer(
            ctx.accounts.asset_program.key(),
            UpdateCardMetadata {
                governance_authority: ctx.accounts.authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                config: ctx.accounts.asset_config.to_account_info(),
                card: ctx.accounts.card.to_account_info(),
                card_mint: ctx.accounts.card_mint.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            signer_seeds,
        ),
        proposal.field,
        proposal.value.clone(),
    )?;

    let proposal = &mut ctx.accounts.proposal;
    proposal.status = ProposalStatus::Executed;
    emit!(ProposalExecuted {
        proposal: proposal.key(),
        yes: proposal.yes,
        no: proposal.no,
    });
    Ok(())
}
