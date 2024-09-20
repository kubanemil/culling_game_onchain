use crate::state::Game;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(game_id: u32)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: opponent is given my signer
    pub opponent: UncheckedAccount<'info>,

    #[account(init, payer=signer, space=Game::INIT_SPACE, 
        seeds=[b"game", &game_id.to_le_bytes()[..], signer.key().as_ref()], bump)]
    pub game: Account<'info, Game>,

    pub system_program: Program<'info, System>
}

impl<'info> CreateGame<'info> {
    pub fn create_game(&mut self, game_id: u32, stake_amount: u64, bumps: CreateGameBumps) -> Result<()> {
        self.game.set_inner(Game {
            id: game_id,
            stake_amount,
            players: [self.signer.key(), self.opponent.key()],
            bump: bumps.game, 
        });
        Ok(())
    }
}
