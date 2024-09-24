use crate::constants::VAULT_ADDRESS;
use crate::state::Vote;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken as AssociatedTokenProgram,
    token::{self, Token as TokenProgram, TokenAccount},
};
use asset::program::Asset;

#[derive(Accounts)]
#[instruction(amendment_id: u32)]
pub struct VoteFor<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer=signer, space=Vote::INIT_SPACE,
        seeds=[b"vote", signer.key().as_ref(), &amendment_id.to_le_bytes()[..]], bump)]
    pub vote: Account<'info, Vote>,

    #[account(mut, seeds=[b"cullingToken", VAULT_ADDRESS.as_ref()], bump)]
    pub mint: Account<'info, token::Mint>,

    #[account(init, payer=signer,
        associated_token::mint=mint, associated_token::authority=vote)]
    pub vote_ata: Account<'info, token::TokenAccount>,

    #[account(mut, associated_token::mint=mint, associated_token::authority=signer)]
    pub signer_ata: Account<'info, TokenAccount>,

    pub asset_program: Program<'info, Asset>,
    pub associated_token_program: Program<'info, AssociatedTokenProgram>,
    pub token_program: Program<'info, TokenProgram>,
    pub system_program: Program<'info, System>,
}

impl<'info> VoteFor<'info> {
    pub fn vote_for(&mut self, amendment_id: u32, accept: bool, tokens: u64) -> Result<()> {
        msg!("Voting for amendment with id: {:?}", amendment_id);

        self.transfer_tokens(tokens)?;

        self.vote.set_inner(Vote {
            voter: self.signer.key(),
            amendment_id,
            accept,
            tokens,
        });

        Ok(())
    }

    pub fn transfer_tokens(&mut self, amount: u64) -> Result<()> {
        // transfer tokens to vote
        let transfer_accounts = token::Transfer {
            from: self.signer_ata.to_account_info(),
            to: self.vote_ata.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), transfer_accounts);
        token::transfer(ctx, amount)?;
        Ok(())
    }
}
