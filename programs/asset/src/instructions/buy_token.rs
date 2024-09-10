use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_spl::{associated_token::AssociatedToken, token::{mint_to, MintTo, Token, TokenAccount}};
use anchor_spl::token_interface::Mint;
use crate::state::AuthVault;

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init_if_needed, payer=signer, associated_token::mint=mint, associated_token::authority=signer)]
    pub signer_ata: Account<'info, TokenAccount>,

    #[account(mut, seeds=[b"emilToken", signer.key.as_ref()], bump)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut, seeds=[b"authVault"], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyToken<'info> {
    pub fn buy_token(&mut self, amount: u64) -> Result<()> {
        let lamports = amount / 100;
        transfer(self.signer.key, &self.vault.key(), lamports);

        let accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.signer_ata.to_account_info(),
            authority: self.vault.to_account_info()
        };

        let seeds = &[&b"authVault"[..], &[self.vault.bump]];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        mint_to(ctx, amount)
    }
}
