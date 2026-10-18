
export const DuelErrorCode = {
  StakeTooSmall: 6000,
  SelfPlay: 6001,
  NotOpen: 6002,
  NotAccepted: 6003,
  WrongOpponent: 6004,
  NotAPlayer: 6005,
  AlreadyRevealed: 6006,
  RevealWindowClosed: 6007,
  RevealWindowOpen: 6008,
  CommitmentMismatch: 6009,
  CardNotOwned: 6010,
  AlreadyClaimed: 6011,
  MissingCardAccounts: 6012,
  WrongCard: 6013,
  MathOverflow: 6014
};

export type DuelErrorName = keyof typeof DuelErrorCode;
