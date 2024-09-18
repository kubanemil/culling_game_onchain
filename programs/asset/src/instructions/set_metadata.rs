use anchor_lang::prelude::*;
use anchor_spl::metadata::mpl_token_metadata::instructions::CreateMetadataAccountV3CpiBuilder;
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
use anchor_spl::metadata::{Metadata, MetadataAccount};
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
#[instruction(card_id: u8)]
pub struct SetMetadata<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(seeds=[b"card", &[card_id][..], signer.key().as_ref()], bump)]
    pub card: Account<'info, token::Mint>,

    /// CHECK: metadata account
    #[account(seeds=[b"metadata", metadata_program.key().as_ref(), card.key().as_ref()], seeds::program=metadata_program.key(), bump)]
    pub metadata: Account<'info, MetadataAccount>,

    pub metadata_program: Program<'info, Metadata>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> SetMetadata<'info> {
    pub fn set_metadata(&mut self, card_id: u8, name: String, uri: String) -> Result<()> {
        msg!("card ID & address: {} | {}", &card_id, self.card.key());
        msg!("metadata address: {}", self.metadata.key());
        let symbol = name.clone() + "@";

        // use metaplex_program to attach metadata to card
        let card_data = DataV2 {
            name,
            symbol: symbol,
            uri,
            seller_fee_basis_points: 1,
            creators: None,
            collection: None,
            uses: None,
        };

        CreateMetadataAccountV3CpiBuilder::new(&self.metadata_program.to_account_info())
            .metadata(&self.metadata.to_account_info())
            .mint(&self.card.to_account_info())
            .mint_authority(&self.signer.to_account_info())
            .payer(&self.signer.to_account_info())
            .update_authority(&self.signer.to_account_info(), false)
            .system_program(&self.system_program.to_account_info())
            .data(card_data)
            .is_mutable(true)
            .invoke()?;
        Ok(())
    }
}
