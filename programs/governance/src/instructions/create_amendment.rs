use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::Amendment};

#[derive(Accounts)]
pub struct CreateAmendement<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer=signer, space=Amendment::INIT_SPACE,
        seeds=[b"amendment", signer.key().as_ref()], bump)]
    pub amendment: Account<'info, Amendment>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateAmendement<'info> {
    pub fn create(
        &mut self,
        card_id: u8,
        new_metadata_uri: String,
        deadline_slot: u64,
    ) -> Result<()> {
        msg!("Greetings from: {:?}", self.signer.key);
        require!(Clock::get()?.slot < deadline_slot, ErrorCode::CustomError);

        self.amendment.set_inner(Amendment {
            creator: self.signer.key(),
            card_id,
            new_metadata_uri,
            deadline_slot,
            pros: 0,
            cons: 0,
        });
        Ok(())
    }
}
