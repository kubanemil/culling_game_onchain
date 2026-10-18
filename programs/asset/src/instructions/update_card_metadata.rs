use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    token_metadata_update_field, Mint, Token2022, TokenMetadataUpdateField,
};

use crate::{
    constants::*,
    error::AssetError,
    events::CardMetadataUpdated,
    instructions::fund_rent,
    state::{Card, CardField, Config},
};

#[derive(Accounts)]
pub struct UpdateCardMetadata<'info> {
    /// Only the governance program can produce this signature, after a proposal passes.
    #[account(address = GOVERNANCE_AUTHORITY)]
    pub governance_authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [CARD_SEED, card_mint.key().as_ref()],
        bump = card.bump,
    )]
    pub card: Account<'info, Card>,

    #[account(mut, address = card.mint)]
    pub card_mint: Box<InterfaceAccount<'info, Mint>>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_update_card_metadata(
    ctx: Context<UpdateCardMetadata>,
    field: CardField,
    value: String,
) -> Result<()> {
    require!(field.accepts(&value), AssetError::InvalidMetadata);

    let accounts = ctx.accounts;
    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[accounts.config.bump]]];
    token_metadata_update_field(
        CpiContext::new_with_signer(
            accounts.token_program.key(),
            TokenMetadataUpdateField {
                program_id: accounts.token_program.to_account_info(),
                metadata: accounts.card_mint.to_account_info(),
                update_authority: accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        field.into(),
        value.clone(),
    )?;

    fund_rent(
        accounts.payer.to_account_info(),
        accounts.card_mint.to_account_info(),
        accounts.system_program.key(),
    )?;

    emit!(CardMetadataUpdated {
        card: accounts.card.key(),
        field,
        value,
    });
    Ok(())
}
