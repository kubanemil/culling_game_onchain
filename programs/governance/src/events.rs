use anchor_lang::prelude::*;

#[event]
pub struct VoteCast {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub approve: bool,
    pub weight: u64,
}

#[event]
pub struct ProposalExecuted {
    pub proposal: Pubkey,
    pub yes: u64,
    pub no: u64,
}
