
export const AssetErrorCode = {
  NotUpgradeAuthority: 6000,
  Unauthorized: 6001,
  InvalidStats: 6002,
  InvalidPrice: 6003,
  InvalidMetadata: 6004,
  MathOverflow: 6005
};

export type AssetErrorName = keyof typeof AssetErrorCode;
