use crate::error::ErrorCode;
use crate::state::{Config, Game, Player};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(game_id: u32)]
pub struct Resolve<'info> {
    #[account(mut)]
    pub auth: Signer<'info>,

    /// CHECK: Creator to retrieve game
    pub creator: UncheckedAccount<'info>,

    /// CHECK: Winner is provided by auth
    #[account(mut)]
    pub winner: UncheckedAccount<'info>,

    #[account(mut, seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,

    #[account(mut, seeds=[b"game", &game_id.to_le_bytes()[..], creator.key().as_ref()], bump)]
    pub game: Account<'info, Game>,

    #[account(mut, seeds=[b"player", game.creator.as_ref()], bump)]
    pub creator_player: Account<'info, Player>,

    #[account(mut, seeds=[b"player", game.opponent.as_ref()], bump)]
    pub opponent_player: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}

impl<'info> Resolve<'info> {
    pub fn resolve(&mut self, game_id: u32) -> Result<()> {
        msg!("Game id: {}", game_id);
        require!(self.auth.key() == self.config.auth, ErrorCode::NotAuthority);
        require!(
            self.winner.key() == self.game.creator || self.winner.key() == self.game.opponent,
            ErrorCode::InvalidWinner
        );

        let (winner, loser) = if self.winner.key() == self.game.creator {
            (&mut self.creator_player, &mut self.opponent_player)
        } else {
            (&mut self.opponent_player, &mut self.creator_player)
        };

        // Update the game counters
        winner.game_won += 1;
        loser.game_lost += 1;

        self.game.close(self.winner.to_account_info())?;
        Ok(())
    }
}
