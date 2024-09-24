use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Custom error message")]
    CustomError,

    #[msg("You cannot vote after deadline")]
    VoteAfterDeadline,

    #[msg("You cannot resolve before deadline")]
    ResolveBeforeDeadline,
}
