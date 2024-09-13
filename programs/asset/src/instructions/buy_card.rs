use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token};
use mpl_core::instructions::CreateV1CpiBuilder;
use mpl_core::programs::MPL_CORE_ID;


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

    /// CHECK: it's new program, no struct for it for now.
    #[account(address=MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyCard<'info> {
    pub fn buy_card(&mut self, card_id: u8) -> Result<()> {
        msg!("card ID: {}", card_id);
        let tokens_amount = 10 * 10u64.pow(6);

        // transfer tokens to vault
        let accounts = token::Transfer {
            from: self.signer_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
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
        token::transfer(ctx, tokens_amount)?;

        // card is already created in validator.

        // associate metadata with card using mpl_core
        CreateV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.card.to_account_info())
            .collection(None)
            .authority(Some(&self.signer.to_account_info()))
            .payer(&self.signer.to_account_info())
            .name("Emil Token".to_owned())
            .uri("https://developers.metaplex.com/core/create-asset".to_string())
            .invoke()?;
        Ok(())
    }
}
