use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateAmendement<'info> {
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateAmendement<'info> {
    pub fn create(&self) -> Result<()> {
        msg!("Greetings from: {:?}", self.signer.key);
        Ok(())
    }
}
