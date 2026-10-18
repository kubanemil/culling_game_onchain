use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{burn, mint_to, Burn, Mint, MintTo, Token2022, TokenAccount},
};

use crate::{
    constants::*,
    state::{Card, Config},
};

#[derive(Accounts)]
pub struct BuyCard<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [CARD_SEED, card_mint.key().as_ref()],
        bump = card.bump,
    )]
    pub card: Account<'info, Card>,

    #[account(mut, address = card.mint)]
    pub card_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut, address = sale::TOKEN_MINT)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = buyer,
        token::token_program = token_program,
    )]
    pub buyer_tokens: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = card_mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
    )]
    pub buyer_cards: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// The price is burned, so there is no treasury to drain.
pub fn handle_buy_card(ctx: Context<BuyCard>) -> Result<()> {
    let accounts = ctx.accounts;
    burn(
        CpiContext::new(
            accounts.token_program.key(),
            Burn {
                mint: accounts.token_mint.to_account_info(),
                from: accounts.buyer_tokens.to_account_info(),
                authority: accounts.buyer.to_account_info(),
            },
        ),
        accounts.card.price,
    )?;

    let signer_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[accounts.config.bump]]];
    mint_to(
        CpiContext::new_with_signer(
            accounts.token_program.key(),
            MintTo {
                mint: accounts.card_mint.to_account_info(),
                to: accounts.buyer_cards.to_account_info(),
                authority: accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )
}
