use anchor_lang::prelude::*;

#[error_code]
pub enum DuelError {
    #[msg("Stake is below the minimum of 0.001 SOL")]
    StakeTooSmall,
    #[msg("A player cannot duel themselves")]
    SelfPlay,
    #[msg("Duel is not open")]
    NotOpen,
    #[msg("Duel was not accepted")]
    NotAccepted,
    #[msg("This duel is reserved for another opponent")]
    WrongOpponent,
    #[msg("Signer is not a player in this duel")]
    NotAPlayer,
    #[msg("Player has already revealed")]
    AlreadyRevealed,
    #[msg("Reveal window has closed")]
    RevealWindowClosed,
    #[msg("Reveal window is still open")]
    RevealWindowOpen,
    #[msg("Card and secret do not match the commitment")]
    CommitmentMismatch,
    #[msg("Player does not hold the revealed card")]
    CardNotOwned,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
