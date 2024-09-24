use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::metadata::Metadata;
use anchor_spl::token::{self, Token};
use asset::cpi::accounts::UpdateMetadata;
use asset::cpi::update_metadata_uri;
use asset::{program::Asset, AuthVault};

use crate::{error::ErrorCode, state::Amendment};

#[derive(Accounts)]
pub struct ResolveAmendment<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"amendment", signer.key().as_ref()], bump)]
    pub amendment: Account<'info, Amendment>,

    #[account(mut)]
    pub card: Account<'info, token::Mint>,

    #[account(mut)]
    pub vault: Account<'info, AuthVault>,

    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub asset_program: Program<'info, Asset>,
    pub metadata_program: Program<'info, Metadata>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

impl<'info> ResolveAmendment<'info> {
    pub fn resolve(&self) -> Result<()> {
        msg!("Greetings from: {:?}", self.signer.key);
        require!(
            Clock::get()?.slot > self.amendment.deadline_slot,
            ErrorCode::ResolveBeforeDeadline
        );

        if self.amendment.pros > self.amendment.cons {
            let accounts = UpdateMetadata {
                signer: self.signer.to_account_info(),
                card: self.card.to_account_info(),
                vault: self.vault.to_account_info(),
                metadata: self.metadata.to_account_info(),
                metadata_program: self.metadata_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
                system_program: self.system_program.to_account_info(),
                sysvar_instructions: self.sysvar_instructions.to_account_info(),
            };
            let ctx = CpiContext::new(self.asset_program.to_account_info(), accounts);

            update_metadata_uri(
                ctx,
                self.amendment.card_id,
                self.amendment.new_metadata_uri.clone(),
            )?;
        } else {
            msg!("Can't update as amendment is not accepted by majority")
        }

        Ok(())
    }
}
