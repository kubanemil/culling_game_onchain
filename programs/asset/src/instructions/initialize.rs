use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, Mint};
use crate::state::AuthVault;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, seeds=[b"emilToken", signer.key.as_ref()], payer=signer, bump, mint::decimals=6, mint::authority=vault)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(init, payer=signer, seeds=[b"authVault"], bump, space=AuthVault::INIT_SPACE)]
    pub vault: Account<'info, AuthVault>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(&mut self, bumps: InitializeBumps) -> Result<()> {
        self.vault.set_inner(
            AuthVault {
                bump: bumps.vault,
                mint_bump: bumps.mint,
            }
        );
        Ok(())
    }
}
