use anchor_lang::prelude::*;

use crate::{
    constants::*,
    error::DuelError,
    events::DuelSettled,
    fight::{fight, Outcome},
    state::{randomness, Duel, DuelStatus},
};

#[derive(Accounts)]
pub struct Settle<'info> {
    #[account(
        mut,
        close = creator,
        has_one = creator,
        seeds = [DUEL_SEED, creator.key().as_ref(), &duel.nonce.to_le_bytes()],
        bump = duel.bump,
        constraint = duel.status == DuelStatus::Accepted @ DuelError::NotAccepted,
        constraint = duel.opponent == Some(opponent.key()) @ DuelError::NotAPlayer,
    )]
    pub duel: Account<'info, Duel>,

    #[account(mut)]
    pub creator: SystemAccount<'info>,

    #[account(mut)]
    pub opponent: SystemAccount<'info>,
}

/// Anyone can settle. A missing reveal forfeits; if neither revealed, both are refunded.
pub fn handle_settle(ctx: Context<Settle>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let duel_key = ctx.accounts.duel.key();
    let duel = &ctx.accounts.duel;

    let outcome = match (&duel.creator_reveal, &duel.opponent_reveal) {
        (Some(creator), Some(opponent)) => fight(
            &creator.stats,
            &opponent.stats,
            &randomness(&duel_key, &creator.secret, &opponent.secret),
        ),
        _ if now <= duel.reveal_deadline => return err!(DuelError::RevealWindowOpen),
        (Some(_), None) => Outcome::CreatorWins,
        (None, Some(_)) => Outcome::OpponentWins,
        (None, None) => Outcome::Draw,
    };

    let pot = duel.stake.checked_mul(2).ok_or(DuelError::MathOverflow)?;
    let (to_opponent, winner) = match outcome {
        Outcome::CreatorWins => (0, Some(duel.creator)),
        Outcome::OpponentWins => (pot, duel.opponent),
        Outcome::Draw => (duel.stake, None),
    };

    emit!(DuelSettled {
        duel: duel_key,
        creator: duel.creator,
        opponent: ctx.accounts.opponent.key(),
        winner,
        stake: duel.stake,
    });

    // The creator's share and the rent go back through `close = creator`.
    if to_opponent > 0 {
        ctx.accounts.duel.sub_lamports(to_opponent)?;
        ctx.accounts.opponent.add_lamports(to_opponent)?;
    }
    Ok(())
}
