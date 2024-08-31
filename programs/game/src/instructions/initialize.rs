use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    pub signer: Signer<'info>
}

impl<'info> Initialize<'info> {
    pub fn init_game(&self) -> Result<()>{
        Ok(())
    }
}