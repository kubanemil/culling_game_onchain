use crate::state::{Game, Player};
use crate::utils::transfer_lamports;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(game_id: u32)]
pub struct Create<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: opponent is given my signer
    pub opponent: UncheckedAccount<'info>,

    #[account(init, payer=signer, space=Game::INIT_SPACE,
        seeds=[b"game", &game_id.to_le_bytes()[..], signer.key().as_ref()], bump)]
    pub game: Account<'info, Game>,

    #[account(init_if_needed, payer=signer, space=Player::INIT_SPACE,
        seeds=[b"player", signer.key().as_ref()], bump)]
    pub player: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}

impl<'info> Create<'info> {
    pub fn create(&mut self, game_id: u32, stake_lamports: u64, bumps: CreateBumps) -> Result<()> {
        // transfer funds to vault
        transfer_lamports(
            self.signer.to_account_info(),
            self.game.to_account_info(),
            stake_lamports,
        )?;

        // set the game info
        self.game.set_inner(Game {
            id: game_id,
            stake_amount: stake_lamports,
            creator: self.signer.key(),
            opponent: self.opponent.key(),
            accepted: false,
            bump: bumps.game,
        });

        if !self.player.initiated {
            self.player.set_inner(Player {
                initiated: true,
                owner: self.signer.key(),
                game_won: 0,
                game_lost: 0,
            });
        }
        Ok(())
    }
}
