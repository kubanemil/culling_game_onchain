
export const SaleErrorCode = {
  NotUpgradeAuthority: 6000,
  Unauthorized: 6001,
  NotPendingAdmin: 6002,
  InvalidDecimals: 6003,
  InvalidSupplyCap: 6004,
  InvalidMetadata: 6005,
  InvalidSchedule: 6006,
  InvalidPricing: 6007,
  InvalidVesting: 6008,
  SupplyCapExceeded: 6009,
  RoundPaused: 6010,
  RoundNotStarted: 6011,
  RoundEnded: 6012,
  RoundNotEnded: 6013,
  ZeroAmount: 6014,
  ProofTooLong: 6015,
  InvalidProof: 6016,
  AllocationExceeded: 6017,
  HardCapExceeded: 6018,
  NothingToClaim: 6019,
  NoProceeds: 6020,
  AlreadyFinalized: 6021,
  MathOverflow: 6022
};

export type SaleErrorName = keyof typeof SaleErrorCode;
