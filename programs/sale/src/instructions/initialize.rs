use anchor_lang::{
    prelude::*,
    solana_program::bpf_loader_upgradeable,
    system_program::{transfer, Transfer},
};
use anchor_spl::token_interface::{
    token_metadata_initialize, Mint, Token2022, TokenMetadataInitialize,
};

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
        valid_len(&args.name, MAX_NAME_LEN)
            && valid_len(&args.symbol, MAX_SYMBOL_LEN)
            && valid_len(&args.uri, MAX_URI_LEN),
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

    // Token-2022 grows the mint to fit the metadata but leaves rent to the caller.
    let mint = accounts.mint.to_account_info();
    let missing = Rent::get()?
        .minimum_balance(mint.data_len())
        .saturating_sub(mint.lamports());
    if missing > 0 {
        transfer(
            CpiContext::new(
                accounts.system_program.key(),
                Transfer {
                    from: accounts.admin.to_account_info(),
                    to: mint,
                },
            ),
            missing,
        )?;
    }
    Ok(())
}

fn valid_len(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max
}
