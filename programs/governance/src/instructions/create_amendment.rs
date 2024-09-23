use anchor_lang::prelude::*;

use crate::state::Amendment;

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
        amendment_id: u32,
        card_id: u8,
        new_metadata: Pubkey,
        deadline_slot: u128,
    ) -> Result<()> {
        msg!("Greetings from: {:?}", self.signer.key);

        self.amendment.set_inner(Amendment {
            id: amendment_id,
            card_id,
            new_metadata,
            deadline_slot,
        });
        Ok(())
    }
}
