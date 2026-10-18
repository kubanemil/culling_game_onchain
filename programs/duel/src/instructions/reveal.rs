use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use asset::{Card, CARD_SEED};

use crate::{
    constants::*,
    error::DuelError,
    state::{commitment, Duel, DuelStatus, Revealed},
};

#[derive(Accounts)]
pub struct Reveal<'info> {
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [DUEL_SEED, duel.creator.as_ref(), &duel.nonce.to_le_bytes()],
        bump = duel.bump,
    )]
    pub duel: Account<'info, Duel>,

    #[account(
        seeds = [CARD_SEED, card.mint.as_ref()],
        bump = card.bump,
        seeds::program = asset::ID,
    )]
    pub card: Account<'info, Card>,

    #[account(
        token::mint = card.mint,
        token::authority = player,
        constraint = player_cards.amount >= 1 @ DuelError::CardNotOwned,
    )]
    pub player_cards: InterfaceAccount<'info, TokenAccount>,
}

pub fn handle_reveal(ctx: Context<Reveal>, secret: [u8; 32]) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let player = ctx.accounts.player.key();
    let card = &ctx.accounts.card;
    let duel_key = ctx.accounts.duel.key();
    let duel = &mut ctx.accounts.duel;

    require!(duel.status == DuelStatus::Accepted, DuelError::NotAccepted);
    require!(now <= duel.reveal_deadline, DuelError::RevealWindowClosed);

    let (expected, slot) = if player == duel.creator {
        (duel.creator_commitment, &mut duel.creator_reveal)
    } else if Some(player) == duel.opponent {
        (duel.opponent_commitment, &mut duel.opponent_reveal)
    } else {
        return err!(DuelError::NotAPlayer);
    };
    require!(slot.is_none(), DuelError::AlreadyRevealed);
    require!(
        commitment(&duel_key, &player, &card.mint, &secret) == expected,
        DuelError::CommitmentMismatch
    );

    *slot = Some(Revealed {
        card: card.mint,
        stats: card.stats,
        secret,
    });
    Ok(())
}
