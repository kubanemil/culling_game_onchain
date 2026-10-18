use anchor_lang::prelude::*;

#[error_code]
pub enum SaleError {
    #[msg("Signer is not the program upgrade authority")]
    NotUpgradeAuthority,
    #[msg("Signer is not the sale admin")]
    Unauthorized,
    #[msg("Signer is not the pending admin")]
    NotPendingAdmin,
    #[msg("Decimals must be at most 9")]
    InvalidDecimals,
    #[msg("Supply cap must be greater than zero")]
    InvalidSupplyCap,
    #[msg("Name, symbol or URI is empty or too long")]
    InvalidMetadata,
    #[msg("Round must end after it starts and in the future")]
    InvalidSchedule,
    #[msg("Price and hard cap must be greater than zero")]
    InvalidPricing,
    #[msg("Cliff must be between zero and the vesting period, which is at most ten years")]
    InvalidVesting,
    #[msg("Round caps would exceed the supply cap")]
    SupplyCapExceeded,
    #[msg("Round is paused")]
    RoundPaused,
    #[msg("Round has not started")]
    RoundNotStarted,
    #[msg("Round has ended")]
    RoundEnded,
    #[msg("Round has not ended")]
    RoundNotEnded,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Merkle proof is longer than the maximum depth")]
    ProofTooLong,
    #[msg("Wallet and allocation are not in the allowlist")]
    InvalidProof,
    #[msg("Purchase exceeds the wallet allocation")]
    AllocationExceeded,
    #[msg("Purchase exceeds the round hard cap")]
    HardCapExceeded,
    #[msg("Nothing has vested since the last claim")]
    NothingToClaim,
    #[msg("Round has no proceeds to withdraw")]
    NoProceeds,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
