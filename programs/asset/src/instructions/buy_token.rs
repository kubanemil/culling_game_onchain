use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init_if_needed, payer=signer, associated_token::mint=mint, associated_token::authority=signer)]
    pub signer_ata: Account<'info, token::TokenAccount>,

    #[account(mut, seeds=[b"emilToken", vault.key().as_ref()], bump=vault.mint_bump)]
    pub mint: Account<'info, token::Mint>,

    #[account(mut, seeds=[b"authVault", signer.key().as_ref()], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyToken<'info> {
    pub fn buy_token(&mut self, amount: u64) -> Result<()> {
        let lamports = amount / 100;
        system_instruction::transfer(self.signer.key, &self.vault.key(), lamports);

        let accounts = token::MintTo {
            mint: self.mint.to_account_info(),
            to: self.signer_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        };

        let seeds = &[
            "authVault".as_bytes(),
            self.signer.key.as_ref(),
            &[self.vault.bump],
        ];
        let signer_seeds = &[seeds.as_slice()];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        token::mint_to(ctx, amount)
    }
}
