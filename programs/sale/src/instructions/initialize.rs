use anchor_lang::{prelude::*, solana_program::bpf_loader_upgradeable};
use anchor_spl::token_interface::{
    spl_pod::optional_keys::OptionalNonZeroPubkey, token_metadata_initialize,
    token_metadata_update_authority, Mint, Token2022, TokenMetadataInitialize,
    TokenMetadataUpdateAuthority,
};
use culling_common::{fund_rent, valid_metadata};

use crate::{constants::*, error::SaleError, state::Config};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeArgs {
    pub decimals: u8,
    pub supply_cap: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
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

    #[account(
        init,
        payer = admin,
        seeds = [MINT_SEED],
        bump,
        mint::decimals = args.decimals,
        mint::authority = config,
        mint::token_program = token_program,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    // Only the upgrade authority can initialize, so nobody can front-run the deploy.
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(admin.key())
            @ SaleError::NotUpgradeAuthority,
    )]
    pub program_data: Account<'info, ProgramData>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
    require!(args.decimals <= MAX_DECIMALS, SaleError::InvalidDecimals);
    require!(args.supply_cap > 0, SaleError::InvalidSupplyCap);
    require!(
        valid_metadata(&args.name, &args.symbol, &args.uri),
        SaleError::InvalidMetadata
    );

    let accounts = ctx.accounts;
    accounts.config.set_inner(Config {
        admin: accounts.admin.key(),
        pending_admin: None,
        mint: accounts.mint.key(),
        decimals: args.decimals,
        supply_cap: args.supply_cap,
        allocated: 0,
        round_count: 0,
        bump: ctx.bumps.config,
    });

    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[ctx.bumps.config]]];
    token_metadata_initialize(
        CpiContext::new_with_signer(
            accounts.token_program.key(),
            TokenMetadataInitialize {
                program_id: accounts.token_program.to_account_info(),
                metadata: accounts.mint.to_account_info(),
                update_authority: accounts.config.to_account_info(),
                mint_authority: accounts.config.to_account_info(),
                mint: accounts.mint.to_account_info(),
            },
            signer_seeds,
        ),
        args.name,
        args.symbol,
        args.uri,
    )?;

    // No instruction edits the token's metadata, so drop the authority and make that final.
    token_metadata_update_authority(
        CpiContext::new_with_signer(
            accounts.token_program.key(),
            TokenMetadataUpdateAuthority {
                program_id: accounts.token_program.to_account_info(),
                metadata: accounts.mint.to_account_info(),
                current_authority: accounts.config.to_account_info(),
                new_authority: accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        OptionalNonZeroPubkey::default(),
    )?;

    fund_rent(
        accounts.admin.to_account_info(),
        accounts.mint.to_account_info(),
        accounts.system_program.key(),
    )
}
