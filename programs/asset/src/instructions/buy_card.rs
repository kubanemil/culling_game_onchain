use crate::error::ErrorCode;
use crate::state::AuthVault;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token};

#[derive(Accounts)]
#[instruction(card_id: u8)]
pub struct BuyCard<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"cullingToken", vault.key().as_ref()], bump=vault.mint_bump)]
    pub mint: Account<'info, token::Mint>,

    #[account(mut, seeds=[b"card", &[card_id][..], vault.owner.as_ref()], bump)]
    pub card: Account<'info, token::Mint>,

    #[account(mut, associated_token::mint=mint, associated_token::authority=signer)]
    pub signer_ata: Account<'info, token::TokenAccount>,

    #[account(init_if_needed, payer=signer, associated_token::mint=card, associated_token::authority=signer)]
    pub signer_card_ata: Account<'info, token::TokenAccount>,

    #[account(mut, seeds=[b"authVault"], bump=vault.bump)]
    pub vault: Account<'info, AuthVault>,

    #[account(mut, associated_token::mint=mint, associated_token::authority=vault)]
    pub vault_ata: Account<'info, token::TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> BuyCard<'info> {
    pub fn buy_card(&mut self, card_id: u8) -> Result<()> {
        msg!("card ID & address: {} | {}", &card_id, self.card.key());
        require!(card_id < 10, ErrorCode::InvalidCardId);

        let tokens_amount = 10 * 10u64.pow(6); // 10 cullingTokens

        // transfer tokens to vault
        let transfer_accounts = token::Transfer {
            from: self.signer_ata.to_account_info(),
            to: self.vault_ata.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);
        token::transfer(ctx, tokens_amount)?;

        // mint card
        let mint_accounts = token::MintTo {
            mint: self.card.to_account_info(),
            to: self.signer_card_ata.to_account_info(),
            authority: self.vault.to_account_info(),
        };

        let seeds = &["authVault".as_bytes(), &[self.vault.bump]];
        let signer_seeds = &[seeds.as_slice()];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            mint_accounts,
            signer_seeds,
        );
        token::mint_to(ctx, 1)
    }
}
