use crate::error::ErrorCode;
use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
#[instruction(card_id: u8)]
pub struct InitCard<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer=signer,mint::decimals=0, mint::authority=vault.key(),
        seeds=[b"card", &[card_id][..], vault.owner.as_ref()], bump)]
    pub card: Account<'info, token::Mint>,

    #[account(mut, seeds=[b"authVault"], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitCard<'info> {
    pub fn init_card(&mut self, card_id: u8) -> Result<()> {
        msg!("Card ID: {}. Card address: {}", card_id, self.card.key());
        require!(card_id < 10, ErrorCode::InvalidCardId);
        require!(self.signer.key() == self.vault.owner, ErrorCode::NotOwner);
        Ok(())
    }
}
