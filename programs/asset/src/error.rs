use anchor_lang::prelude::*;

#[error_code]
pub enum AssetError {
    #[msg("Signer is not the program upgrade authority")]
    NotUpgradeAuthority,
    #[msg("Signer is not the asset admin")]
    Unauthorized,
    #[msg("Attack and health must be 1-100, defense 0-100")]
    InvalidStats,
    #[msg("Card price must be greater than zero")]
    InvalidPrice,
    #[msg("Name, symbol or URI is empty or too long")]
    InvalidMetadata,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
