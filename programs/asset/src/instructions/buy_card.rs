use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
#[instruction(card_id: u8)]
pub struct BuyCard<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"cullingToken", vault.key().as_ref()], bump=vault.mint_bump)]
    pub mint: Account<'info, token::Mint>,

    #[account(init, payer=signer, seeds=[b"card", &[card_id][..], signer.key().as_ref()], bump, mint::decimals=0, mint::authority=signer)]
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

impl<'info> BuyCard<'info> {
    pub fn buy_card(&mut self, card_id: u8) -> Result<()> {
        msg!("card ID & address: {} | {}", &card_id, self.card.key());
        let tokens_amount = 10 * 10u64.pow(6); // 10 cullingTokens

        // transfer tokens to vault
        let accounts = token::Transfer {
            from: self.signer_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), accounts);
        token::transfer(ctx, tokens_amount)?;

        // card account is already created in validator.
        Ok(())
    }
}
