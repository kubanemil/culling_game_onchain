use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
#[instruction(card_id: u8)]
pub struct SetMetadata<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"cullingToken", vault.key().as_ref()], bump=vault.mint_bump)]
    pub mint: Account<'info, token::Mint>,

    #[account(mut, seeds=[b"card", &[card_id][..], signer.key().as_ref()], bump)]
    pub card: Account<'info, token::Mint>,

    #[account(init_if_needed, payer=signer, associated_token::mint=mint, associated_token::authority=signer)]
    pub signer_ata: Account<'info, token::TokenAccount>,

    #[account(mut, seeds=[b"authVault", signer.key().as_ref()], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    #[account(mut, associated_token::mint=mint, associated_token::authority=vault)]
    pub vault_ata: Account<'info, token::TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> SetMetadata<'info> {
    pub fn set_metadata(&mut self, card_id: u8) -> Result<()> {
        msg!("card ID & address: {} | {}", &card_id, self.card.key());
        // use metaplex_program to attach metadata to card

        Ok(())
    }
}
