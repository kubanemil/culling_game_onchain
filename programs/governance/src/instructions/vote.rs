use anchor_lang::prelude::*;

use crate::state::Vote;

#[derive(Accounts)]
#[instruction(amendment_id: u32)]
pub struct VoteFor<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer=signer, space=Vote::INIT_SPACE,
        seeds=[b"vote", signer.key().as_ref(), &amendment_id.to_le_bytes()[..]], bump)]
    pub vote: Account<'info, Vote>,

    pub system_program: Program<'info, System>,
}

impl<'info> VoteFor<'info> {
    pub fn vote_for(&mut self, amendment_id: u32, accept: bool, tokens: u64) -> Result<()> {
        msg!("Voting for amendment with id: {:?}", amendment_id);

        todo!("transfer tokens to vote pda");

        self.vote.set_inner(Vote {
            voter: self.signer.key(),
            amendment_id,
            accept,
            tokens,
        });

        Ok(())
    }
}
