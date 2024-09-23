use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Vote<'info> {
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Vote<'info> {
    pub fn vote(&self) -> Result<()> {
        msg!("Greetings from: {:?}", self.signer.key);
        Ok(())
    }
}
