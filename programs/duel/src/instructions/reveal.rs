use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, Token2022, TokenAccount, TransferChecked,
};
use asset::{Card, CARD_SEED};

use crate::{
    constants::*,
    error::DuelError,
    state::{commitment, Duel, DuelStatus, Revealed},
};

#[derive(Accounts)]
pub struct Reveal<'info> {
    #[account(mut)]
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

    #[account(address = card.mint)]
    pub card_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = card_mint,
        token::authority = player,
        token::token_program = token_program,
        constraint = player_cards.amount >= 1 @ DuelError::CardNotOwned,
    )]
    pub player_cards: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The card stays here until claim, so one card cannot back two reveals or be
    /// borrowed for a single transaction. Escrowing at commit would show the card early.
    #[account(
        init,
        payer = player,
        seeds = [ESCROW_SEED, duel.key().as_ref(), player.key().as_ref()],
        bump,
        token::mint = card_mint,
        token::authority = duel,
        token::token_program = token_program,
    )]
    pub escrow: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handle_reveal(ctx: Context<Reveal>, secret: [u8; 32]) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let player = ctx.accounts.player.key();
    let card_mint = ctx.accounts.card.mint;
    let stats = ctx.accounts.card.stats;
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
        commitment(&duel_key, &player, &card_mint, &secret) == expected,
        DuelError::CommitmentMismatch
    );
    *slot = Some(Revealed {
        card: card_mint,
        stats,
        secret,
    });

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.player_cards.to_account_info(),
                mint: ctx.accounts.card_mint.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        ),
        1,
        ctx.accounts.card_mint.decimals,
    )
}
