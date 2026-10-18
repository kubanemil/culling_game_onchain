use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::*,
    error::SaleError,
    events::Purchased,
    math::purchase_cost,
    merkle,
    state::{Allocation, Config, Round},
};

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [ROUND_SEED, &round.id.to_le_bytes()],
        bump = round.bump,
    )]
    pub round: Account<'info, Round>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = 8 + Allocation::INIT_SPACE,
        seeds = [ALLOCATION_SEED, round.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub allocation: Account<'info, Allocation>,

    pub system_program: Program<'info, System>,
}

pub fn handle_buy(
    ctx: Context<Buy>,
    amount: u64,
    max_amount: u64,
    proof: Vec<[u8; 32]>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let accounts = ctx.accounts;
    let buyer = accounts.buyer.key();
    let round_key = accounts.round.key();
    let round = &accounts.round;

    require!(!round.paused, SaleError::RoundPaused);
    require!(now >= round.start_ts, SaleError::RoundNotStarted);
    require!(now < round.end_ts, SaleError::RoundEnded);
    require!(amount > 0, SaleError::ZeroAmount);
    require!(proof.len() <= MAX_PROOF_LEN, SaleError::ProofTooLong);
    require!(
        merkle::verify(&proof, &round.merkle_root, merkle::leaf(&buyer, max_amount)),
        SaleError::InvalidProof
    );

    let allocation = &mut accounts.allocation;
    if allocation.buyer == Pubkey::default() {
        allocation.set_inner(Allocation {
            round: round_key,
            buyer,
            max_amount,
            purchased: 0,
            claimed: 0,
            bump: ctx.bumps.allocation,
        });
    }

    // Counted in this PDA, not read from a balance, so moving tokens out frees nothing.
    let purchased = allocation
        .purchased
        .checked_add(amount)
        .ok_or(SaleError::MathOverflow)?;
    require!(
        purchased <= allocation.max_amount,
        SaleError::AllocationExceeded
    );
    let sold = round
        .sold
        .checked_add(amount)
        .ok_or(SaleError::MathOverflow)?;
    require!(sold <= round.hard_cap, SaleError::HardCapExceeded);

    let cost = purchase_cost(amount, round.price, accounts.config.decimals)?;
    let proceeds = round
        .proceeds
        .checked_add(cost)
        .ok_or(SaleError::MathOverflow)?;

    transfer(
        CpiContext::new(
            accounts.system_program.key(),
            Transfer {
                from: accounts.buyer.to_account_info(),
                to: accounts.round.to_account_info(),
            },
        ),
        cost,
    )?;

    accounts.allocation.purchased = purchased;
    let round = &mut accounts.round;
    round.sold = sold;
    round.proceeds = proceeds;

    emit!(Purchased {
        round: round_key,
        buyer,
        amount,
        cost,
    });
    Ok(())
}
