use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::metadata::{
    mpl_token_metadata::{
        instructions::{CreateV1CpiBuilder, UpdateV1CpiBuilder},
        types::{Creator, Data, TokenStandard},
    },
    Metadata as MetadataProgram,
};
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
#[instruction(card_id: u8)]
pub struct SetMetadata<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"card", &[card_id][..], vault.owner.as_ref()], bump)]
    pub card: Account<'info, token::Mint>,

    #[account(mut, seeds=[b"authVault"], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    pub metadata_program: Program<'info, MetadataProgram>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: account constraints checked in account trait
    #[account(address = sysvar::instructions::id())]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

impl<'info> SetMetadata<'info> {
    pub fn create_metadata(
        &mut self,
        card_id: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        msg!("card ID & address: {} | {}", &card_id, self.card.key());

        let seeds = &["authVault".as_bytes(), &[self.vault.bump]];
        let vault_seeds = &[seeds.as_slice()];

        // create metadata for card
        CreateV1CpiBuilder::new(&self.metadata_program)
            .metadata(&self.metadata.to_account_info())
            .name(name)
            .uri(uri)
            .symbol(symbol)
            .seller_fee_basis_points(0)
            .mint(&self.card.to_account_info(), false)
            .authority(&self.vault.to_account_info())
            .update_authority(&self.vault.to_account_info(), true)
            .token_standard(TokenStandard::FungibleAsset)
            .system_program(&self.system_program.to_account_info())
            .sysvar_instructions(&self.sysvar_instructions.to_account_info())
            .payer(&self.signer.to_account_info())
            .invoke_signed(vault_seeds)?;
        Ok(())
    }

    pub fn update_metadata(
        &mut self,
        card_id: u8,
        new_name: String,
        new_symbol: String,
        new_uri: String,
    ) -> Result<()> {
        msg!("card ID & address: {} | {}", &card_id, self.card.key());
        msg!("metadata address: {}", &self.metadata.key());

        let seeds = &["authVault".as_bytes(), &[self.vault.bump]];
        let vault_seeds = &[seeds.as_slice()];

        // use metaplex_program to attach metadata to card
        let creators = Some(vec![Creator {
            address: self.vault.key(),
            verified: false,
            share: 100,
        }]);
        let card_data = Data {
            name: new_name,
            symbol: new_symbol,
            uri: new_uri,
            seller_fee_basis_points: 10,
            creators,
        };

        UpdateV1CpiBuilder::new(&self.metadata_program)
            .data(card_data)
            .mint(&self.card.to_account_info())
            .metadata(&self.metadata.to_account_info())
            .authority(&self.vault.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .sysvar_instructions(&self.sysvar_instructions.to_account_info())
            .payer(&self.signer.to_account_info())
            .invoke_signed(vault_seeds)?;
        Ok(())
    }
}
