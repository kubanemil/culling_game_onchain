use crate::error::ErrorCode;
use crate::state::Game;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(game_id: u32)]
pub struct AcceptGame<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: opponent is given my signer
    pub initiator: UncheckedAccount<'info>,

    #[account(mut, seeds=[b"game", &game_id.to_le_bytes()[..], initiator.key().as_ref()], bump)]
    pub game: Account<'info, Game>,

    pub system_program: Program<'info, System>,
}

impl<'info> AcceptGame<'info> {
    pub fn accept_game(&mut self, game_id: u32) -> Result<()> {
        msg!("Game id: {}", game_id);
        require!(
            self.game.players[1] == self.signer.key(),
            ErrorCode::NotGamePlayer
        );

        self.game.accepted = true;
        Ok(())
    }
}
