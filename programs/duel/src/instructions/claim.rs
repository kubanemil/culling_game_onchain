use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, Token2022, TokenAccount, TransferChecked,
};

use crate::{
    constants::*,
    error::DuelError,
    events::DuelSettled,
    fight::Outcome,
    state::{Duel, DuelStatus},
};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [DUEL_SEED, duel.creator.as_ref(), &duel.nonce.to_le_bytes()],
        bump = duel.bump,
        constraint = duel.status == DuelStatus::Accepted @ DuelError::NotAccepted,
    )]
    pub duel: Account<'info, Duel>,

    /// The next three are needed only by a player who revealed, to get the card back.
    pub card_mint: Option<Box<InterfaceAccount<'info, Mint>>>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, duel.key().as_ref(), player.key().as_ref()],
        bump,
    )]
    pub escrow: Option<Box<InterfaceAccount<'info, TokenAccount>>>,

    #[account(
        mut,
        token::mint = card_mint,
        token::authority = player,
        token::token_program = token_program,
    )]
    pub player_cards: Option<Box<InterfaceAccount<'info, TokenAccount>>>,

    pub token_program: Program<'info, Token2022>,
}

/// Each player pulls their own share and card, so nothing is ever pushed to a wallet the
/// other side controls. The creator's claim closes the duel once the opponent is done.
pub fn handle_claim(ctx: Context<Claim>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let player = ctx.accounts.player.key();
    let duel_key = ctx.accounts.duel.key();
    let duel = &ctx.accounts.duel;

    let is_creator = player == duel.creator;
    require!(
        is_creator || Some(player) == duel.opponent,
        DuelError::NotAPlayer
    );
    let outcome = duel.outcome(&duel_key, now)?;
    let (creator_share, opponent_share) = duel.payouts(outcome)?;
    let (claimed, share, revealed) = if is_creator {
        (duel.creator_claimed, creator_share, duel.creator_reveal)
    } else {
        (duel.opponent_claimed, opponent_share, duel.opponent_reveal)
    };
    let opponent_done =
        duel.opponent_claimed || (opponent_share == 0 && duel.opponent_reveal.is_none());

    if !duel.creator_claimed && !duel.opponent_claimed {
        emit!(DuelSettled {
            duel: duel_key,
            creator: duel.creator,
            opponent: duel.opponent.unwrap_or_default(),
            winner: match outcome {
                Outcome::CreatorWins => Some(duel.creator),
                Outcome::OpponentWins => duel.opponent,
                Outcome::Draw => None,
            },
            stake: duel.stake,
        });
    }

    if claimed {
        require!(is_creator && opponent_done, DuelError::AlreadyClaimed);
    } else {
        if let Some(revealed) = revealed {
            return_card(&ctx, revealed.card)?;
        }
        if share > 0 {
            ctx.accounts.duel.sub_lamports(share)?;
            ctx.accounts.player.add_lamports(share)?;
        }
        let duel = &mut ctx.accounts.duel;
        if is_creator {
            duel.creator_claimed = true;
        } else {
            duel.opponent_claimed = true;
        }
    }

    if is_creator && opponent_done {
        ctx.accounts
            .duel
            .close(ctx.accounts.player.to_account_info())?;
    }
    Ok(())
}

fn return_card(ctx: &Context<Claim>, card: Pubkey) -> Result<()> {
    let accounts = &ctx.accounts;
    let (Some(mint), Some(escrow), Some(to)) = (
        &accounts.card_mint,
        &accounts.escrow,
        &accounts.player_cards,
    ) else {
        return err!(DuelError::MissingCardAccounts);
    };
    require_keys_eq!(mint.key(), card, DuelError::WrongCard);

    let duel = &accounts.duel;
    let nonce = duel.nonce.to_le_bytes();
    let signer_seeds: &[&[&[u8]]] = &[&[DUEL_SEED, duel.creator.as_ref(), &nonce, &[duel.bump]]];
    transfer_checked(
        CpiContext::new_with_signer(
            accounts.token_program.key(),
            TransferChecked {
                from: escrow.to_account_info(),
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority: duel.to_account_info(),
            },
            signer_seeds,
        ),
        1,
        mint.decimals,
    )?;
    close_account(CpiContext::new_with_signer(
        accounts.token_program.key(),
        CloseAccount {
            account: escrow.to_account_info(),
            destination: accounts.player.to_account_info(),
            authority: duel.to_account_info(),
        },
        signer_seeds,
    ))
}
