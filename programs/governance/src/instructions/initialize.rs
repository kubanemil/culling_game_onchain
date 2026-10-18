use anchor_lang::{prelude::*, solana_program::bpf_loader_upgradeable};
use anchor_spl::token_interface::Mint;

use crate::{constants::*, error::GovernanceError, state::Config};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeArgs {
    pub quorum: u64,
    pub threshold_bps: u16,
    pub voting_period: i64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump,
    )]
    pub config: Account<'info, Config>,

    /// Voting power is only ever counted in the game token.
    #[account(address = sale::TOKEN_MINT)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(admin.key())
            @ GovernanceError::NotUpgradeAuthority,
    )]
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
    require!(args.quorum > 0, GovernanceError::InvalidQuorum);
    require!(
        (MIN_THRESHOLD_BPS..=MAX_THRESHOLD_BPS).contains(&args.threshold_bps),
        GovernanceError::InvalidThreshold
    );
    require!(
        (MIN_VOTING_PERIOD..=MAX_VOTING_PERIOD).contains(&args.voting_period),
        GovernanceError::InvalidVotingPeriod
    );

    ctx.accounts.config.set_inner(Config {
        mint: ctx.accounts.mint.key(),
        quorum: args.quorum,
        threshold_bps: args.threshold_bps,
        voting_period: args.voting_period,
        proposal_count: 0,
        bump: ctx.bumps.config,
    });
    Ok(())
}
