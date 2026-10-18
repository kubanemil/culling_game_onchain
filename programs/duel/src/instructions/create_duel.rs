use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::*,
    error::DuelError,
    state::{Duel, DuelStatus},
};

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct CreateDuel<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + Duel::INIT_SPACE,
        seeds = [DUEL_SEED, creator.key().as_ref(), &nonce.to_le_bytes()],
        bump,
    )]
    pub duel: Account<'info, Duel>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_duel(
    ctx: Context<CreateDuel>,
    nonce: u64,
    stake: u64,
    commitment: [u8; 32],
    opponent: Option<Pubkey>,
) -> Result<()> {
    require!(stake >= MIN_STAKE, DuelError::StakeTooSmall);
    let creator = ctx.accounts.creator.key();
    require!(opponent != Some(creator), DuelError::SelfPlay);

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.duel.to_account_info(),
            },
        ),
        stake,
    )?;

    ctx.accounts.duel.set_inner(Duel {
        creator,
        nonce,
        opponent,
        stake,
        status: DuelStatus::Open,
        creator_commitment: commitment,
        opponent_commitment: [0; 32],
        reveal_deadline: 0,
        creator_reveal: None,
        opponent_reveal: None,
        creator_claimed: false,
        opponent_claimed: false,
        bump: ctx.bumps.duel,
    });
    Ok(())
}
