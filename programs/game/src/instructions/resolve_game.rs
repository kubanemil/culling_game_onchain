use crate::error::ErrorCode;
use crate::state::Game;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(game_id: u32)]
pub struct ResolveGame<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, seeds=[b"game", &game_id.to_le_bytes()[..], signer.key().as_ref()], bump)]
    pub game: Account<'info, Game>,

    pub system_program: Program<'info, System>,
}

impl<'info> ResolveGame<'info> {
    pub fn resolve_game(&mut self, game_id: u32) -> Result<()> {
        msg!("Game id: {}", game_id);
        require!(
            self.game.players[0] == self.signer.key(),
            ErrorCode::NotGameCreator
        );

        self.game.close(self.signer.to_account_info())?;
        Ok(())
    }
}
