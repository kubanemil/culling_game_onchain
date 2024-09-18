use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::metadata::mpl_token_metadata::instructions::UpdateV1CpiBuilder;
use anchor_spl::metadata::mpl_token_metadata::types::{Creator, Data};
use anchor_spl::metadata::{Metadata as MetadataProgram, MetadataAccount};
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
#[instruction(card_id: u8)]
pub struct SetMetadata<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"card", &[card_id][..], signer.key().as_ref()], bump)]
    pub card: Account<'info, token::Mint>,

    #[account(mut, seeds=[b"authVault", signer.key().as_ref()], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    #[account(mut)]
    pub metadata: Account<'info, MetadataAccount>,

    pub metadata_program: Program<'info, MetadataProgram>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

impl<'info> SetMetadata<'info> {
    pub fn set_metadata(&mut self, card_id: u8, new_name: String, new_uri: String) -> Result<()> {
        msg!("card ID & address: {} | {}", &card_id, self.card.key());
        msg!("metadata address: {}", &self.metadata.key());

        // use metaplex_program to attach metadata to card
        let creators = Some(vec![Creator {
            address: self.vault.key(),
            verified: false,
            share: 100,
        }]);
        let card_data = Data {
            name: new_name,
            symbol: self.metadata.symbol.clone(),
            uri: new_uri,
            seller_fee_basis_points: 10,
            creators,
        };

        UpdateV1CpiBuilder::new(&self.metadata_program)
            .metadata(&self.metadata.to_account_info())
            .data(card_data)
            .mint(&self.card.to_account_info())
            .authority(&self.signer.to_account_info())
            .new_update_authority(self.vault.key())
            .system_program(&self.system_program.to_account_info())
            .sysvar_instructions(&self.sysvar_instructions.to_account_info())
            .payer(&self.signer.to_account_info())
            .invoke()?;
        Ok(())
    }
}
