use anchor_lang::prelude::*;

#[error_code]
pub enum GovernanceError {
    #[msg("Signer is not the program upgrade authority")]
    NotUpgradeAuthority,
    #[msg("Quorum must be greater than zero")]
    InvalidQuorum,
    #[msg("Threshold must be between 5000 and 9999 basis points")]
    InvalidThreshold,
    #[msg("Voting period must be between one minute and thirty days")]
    InvalidVotingPeriod,
    #[msg("Value is empty or too long for this field")]
    InvalidValue,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Escrow holds no voting power")]
    NoVotingPower,
    #[msg("Voting has ended")]
    VotingEnded,
    #[msg("Voting has not ended")]
    VotingNotEnded,
    #[msg("Tokens are locked until the proposals you voted on end")]
    TokensLocked,
    #[msg("Proposal was already executed")]
    AlreadyExecuted,
    #[msg("Proposal passed but its execution window has closed")]
    ProposalExpired,
    #[msg("A newer proposal already changed this field")]
    StaleProposal,
    #[msg("Proposal did not reach quorum")]
    QuorumNotReached,
    #[msg("Proposal did not reach the approval threshold")]
    ThresholdNotReached,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
