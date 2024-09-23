use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ResolveAmendment<'info> {
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ResolveAmendment<'info> {
    pub fn resolve(&self) -> Result<()> {
        msg!("Greetings from: {:?}", self.signer.key);
        Ok(())
    }
}
