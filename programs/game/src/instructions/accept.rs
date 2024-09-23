use crate::error::ErrorCode;
use crate::state::{Game, Player};
use crate::utils::transfer_lamports;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(game_id: u32)]
pub struct Accept<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: opponent is given my signer
    pub initiator: UncheckedAccount<'info>,

    #[account(mut, seeds=[b"game", &game_id.to_le_bytes()[..], initiator.key().as_ref()], bump)]
    pub game: Account<'info, Game>,

    #[account(init_if_needed, payer=signer, space=Player::INIT_SPACE, 
        seeds=[b"player", signer.key().as_ref()], bump)]
    pub player: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}

impl<'info> Accept<'info> {
    pub fn accept(&mut self, game_id: u32) -> Result<()> {
        msg!("Game id: {}", game_id);
        require!(
            self.game.opponent == self.signer.key(),
            ErrorCode::NotGamePlayer
        );

        transfer_lamports(
            self.signer.to_account_info(),
            self.game.to_account_info(),
            self.game.stake_amount,
        )?;

        self.game.accepted = true;

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
