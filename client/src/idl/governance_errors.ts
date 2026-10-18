
export const GovernanceErrorCode = {
  NotUpgradeAuthority: 6000,
  InvalidQuorum: 6001,
  InvalidThreshold: 6002,
  InvalidVotingPeriod: 6003,
  InvalidValue: 6004,
  ZeroAmount: 6005,
  NoVotingPower: 6006,
  VotingEnded: 6007,
  VotingNotEnded: 6008,
  TokensLocked: 6009,
  AlreadyExecuted: 6010,
  ProposalExpired: 6011,
  StaleProposal: 6012,
  QuorumNotReached: 6013,
  ThresholdNotReached: 6014,
  MathOverflow: 6015
};

export type GovernanceErrorName = keyof typeof GovernanceErrorCode;
