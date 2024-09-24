use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program, system_instruction};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"cullingToken", vault.key().as_ref()], bump=vault.mint_bump)]
    pub mint: Account<'info, token::Mint>,

    #[account(init_if_needed, payer=signer, associated_token::mint=mint, associated_token::authority=signer)]
    pub signer_ata: Account<'info, token::TokenAccount>,

    #[account(mut, seeds=[b"authVault"], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyToken<'info> {
    pub fn buy_token(&mut self, amount: u64) -> Result<()> {
        let lamports = amount * 1000;

        // transfer funds to vault
        let transfer_ix =
            system_instruction::transfer(self.signer.key, &self.vault.key(), lamports);
        program::invoke(
            &transfer_ix,
            &[
                self.signer.to_account_info(),
                self.vault.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        // mint tokens to user
        let accounts = token::MintTo {
            mint: self.mint.to_account_info(),
            to: self.signer_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        };

        let seeds = &[
            "authVault".as_bytes(),
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
