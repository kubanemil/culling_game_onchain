use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateGame<'info> {
    pub singer: Signer<'info>,
}

impl<'info> CreateGame<'info> {
    pub fn create_game(&self) -> Result<()> {
        Ok(())
    }
}
