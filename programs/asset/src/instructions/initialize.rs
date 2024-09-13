use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, seeds=[b"cullingToken", vault.key().as_ref()], payer=signer, bump, mint::decimals=6, mint::authority=vault)]
    pub mint: Account<'info, token::Mint>,

    #[account(init, payer=signer, seeds=[b"authVault", signer.key().as_ref()], bump, space=AuthVault::INIT_SPACE)]
    pub vault: Account<'info, AuthVault>,

    #[account(init, payer=signer, associated_token::mint=mint, associated_token::authority=vault)]
    pub vault_ata: Account<'info, token::TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(&mut self, bumps: InitializeBumps) -> Result<()> {
        msg!("Vault, Mint: {} | {}", self.vault.key(), self.mint.key());
        self.vault.set_inner(AuthVault {
            bump: bumps.vault,
            mint_bump: bumps.mint,
        });
        Ok(())
    }
}
