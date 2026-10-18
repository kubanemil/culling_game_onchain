use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    token_metadata_initialize, Mint, Token2022, TokenMetadataInitialize,
};

use culling_common::{fund_rent, valid_metadata};

use crate::{
    constants::*,
    error::AssetError,
    state::{Card, Config, Stats},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateCardArgs {
    pub stats: Stats,
    pub price: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Accounts)]
pub struct CreateCard<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = admin @ AssetError::Unauthorized,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = admin,
        seeds = [CARD_MINT_SEED, &config.card_count.to_le_bytes()],
        bump,
        mint::decimals = 0,
        mint::authority = config,
        mint::token_program = token_program,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = admin,
        space = 8 + Card::INIT_SPACE,
        seeds = [CARD_SEED, mint.key().as_ref()],
        bump,
    )]
    pub card: Account<'info, Card>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_card(ctx: Context<CreateCard>, args: CreateCardArgs) -> Result<()> {
    require!(args.stats.is_valid(), AssetError::InvalidStats);
    require!(args.price > 0, AssetError::InvalidPrice);
    require!(
        valid_metadata(&args.name, &args.symbol, &args.uri),
        AssetError::InvalidMetadata
    );

    let accounts = ctx.accounts;
    let id = accounts.config.card_count;
    accounts.card.set_inner(Card {
        id,
        mint: accounts.mint.key(),
        stats: args.stats,
        price: args.price,
        bump: ctx.bumps.card,
    });
    accounts.config.card_count = id.checked_add(1).ok_or(AssetError::MathOverflow)?;

    // The config PDA is the metadata update authority, so only this program can edit it.
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[accounts.config.bump]]];
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

    fund_rent(
        accounts.admin.to_account_info(),
        accounts.mint.to_account_info(),
        accounts.system_program.key(),
    )
}
