use anchor_lang::prelude::*;

#[event]
pub struct Purchased {
    pub round: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
    pub cost: u64,
}

#[event]
pub struct Claimed {
    pub round: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
}
