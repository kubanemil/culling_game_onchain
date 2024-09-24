use crate::error::ErrorCode;
use crate::state::{Amendment, Vote};
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

    #[account(mut)]
    pub mint: Account<'info, token::Mint>,

    #[account(mut, associated_token::mint=mint, associated_token::authority=signer)]
    pub signer_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub amendment: Account<'info, Amendment>,

    #[account(init, payer=signer, space=Vote::INIT_SPACE,
        seeds=[b"vote", amendment.key().as_ref()], bump)]
    pub vote: Account<'info, Vote>,

    #[account(init, payer=signer,
        associated_token::mint=mint, associated_token::authority=vote)]
    pub vote_ata: Account<'info, token::TokenAccount>,

    pub asset_program: Program<'info, Asset>,
    pub associated_token_program: Program<'info, AssociatedTokenProgram>,
    pub token_program: Program<'info, TokenProgram>,
    pub system_program: Program<'info, System>,
}

impl<'info> VoteFor<'info> {
    pub fn vote_for(&mut self, accept: bool, tokens: u64) -> Result<()> {
        msg!("Voting for amendment with id: {:?}", self.amendment.key());
        require!(
            Clock::get()?.slot < self.amendment.deadline_slot,
            ErrorCode::CustomError
        );
        self.transfer_tokens(tokens)?;

        self.vote.set_inner(Vote {
            voter: self.signer.key(),
            amendment: self.amendment.key(),
            accept,
            tokens,
        });

        match accept {
            true => self.amendment.pros += tokens,
            false => self.amendment.cons += tokens,
        };

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
