
export const AssetErrorCode = {
  NotUpgradeAuthority: 6000,
  Unauthorized: 6001,
  NotPendingAdmin: 6002,
  InvalidStats: 6003,
  InvalidPrice: 6004,
  InvalidMetadata: 6005,
  MathOverflow: 6006
};

export type AssetErrorName = keyof typeof AssetErrorCode;
