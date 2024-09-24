use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("No such card id")]
    InvalidCardId,

    #[msg("You are not an owner of vault")]
    NotOwner,
}
