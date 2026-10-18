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
pub struct AcceptDuel<'info> {
    #[account(mut)]
    pub opponent: Signer<'info>,

    #[account(
        mut,
        seeds = [DUEL_SEED, duel.creator.as_ref(), &duel.nonce.to_le_bytes()],
        bump = duel.bump,
    )]
    pub duel: Account<'info, Duel>,

    pub system_program: Program<'info, System>,
}

pub fn handle_accept_duel(ctx: Context<AcceptDuel>, commitment: [u8; 32]) -> Result<()> {
    let opponent = ctx.accounts.opponent.key();
    let duel = &ctx.accounts.duel;
    require!(duel.status == DuelStatus::Open, DuelError::NotOpen);
    require_keys_neq!(opponent, duel.creator, DuelError::SelfPlay);
    if let Some(invited) = duel.opponent {
        require_keys_eq!(opponent, invited, DuelError::WrongOpponent);
    }
    let reveal_deadline = Clock::get()?
        .unix_timestamp
        .checked_add(REVEAL_WINDOW)
        .ok_or(DuelError::MathOverflow)?;

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.opponent.to_account_info(),
                to: ctx.accounts.duel.to_account_info(),
            },
        ),
        duel.stake,
    )?;

    let duel = &mut ctx.accounts.duel;
    duel.opponent = Some(opponent);
    duel.opponent_commitment = commitment;
    duel.status = DuelStatus::Accepted;
    duel.reveal_deadline = reveal_deadline;
    Ok(())
}
