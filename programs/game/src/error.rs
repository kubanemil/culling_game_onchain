use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,
    #[msg("You are not game's participant")]
    NotGamePlayer,
    #[msg("You are not game's creator")]
    NotGameCreator,
    #[msg("This instruction should be invoked only by auth")]
    NotAuthority,
}
