/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/sale.json`.
 */
export type Sale = {
  "address": "64B6Rmfe4Jg5R4xdPCG3mHpaSihYJLyswC7dPutNQyjo",
  "metadata": {
    "name": "sale",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Token sale: merkle allowlist, capped rounds, vesting"
  },
  "instructions": [
    {
      "name": "acceptAdmin",
      "discriminator": [
        112,
        42,
        45,
        90,
        116,
        181,
        13,
        170
      ],
      "accounts": [
        {
          "name": "newAdmin",
          "signer": true
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "buy",
      "discriminator": [
        102,
        6,
        61,
        18,
        1,
        218,
        235,
        234
      ],
      "accounts": [
        {
          "name": "buyer",
          "writable": true,
          "signer": true
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "round",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  111,
                  117,
                  110,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "round.id",
                "account": "round"
              }
            ]
          }
        },
        {
          "name": "allocation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  108,
                  108,
                  111,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "round"
              },
              {
                "kind": "account",
                "path": "buyer"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "maxAmount",
          "type": "u64"
        },
        {
          "name": "proof",
          "type": {
            "vec": {
              "array": [
                "u8",
                32
              ]
            }
          }
        }
      ]
    },
    {
      "name": "claim",
      "discriminator": [
        62,
        198,
        214,
        193,
        213,
        159,
        108,
        210
      ],
      "accounts": [
        {
          "name": "buyer",
          "writable": true,
          "signer": true,
          "relations": [
            "allocation"
          ]
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "mint",
          "writable": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "round",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  111,
                  117,
                  110,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "round.id",
                "account": "round"
              }
            ]
          },
          "relations": [
            "allocation"
          ]
        },
        {
          "name": "allocation",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  108,
                  108,
                  111,
                  99,
                  97,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "round"
              },
              {
                "kind": "account",
                "path": "buyer"
              }
            ]
          }
        },
        {
          "name": "buyerTokens",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "buyer"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "createRound",
      "discriminator": [
        229,
        218,
        236,
        169,
        231,
        80,
        134,
        112
      ],
      "accounts": [
        {
          "name": "admin",
          "writable": true,
          "signer": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "round",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  111,
                  117,
                  110,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "config.roundCount",
                "account": "config"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "roundArgs"
            }
          }
        }
      ]
    },
    {
      "name": "initialize",
      "discriminator": [
        175,
        175,
        109,
        31,
        13,
        152,
        155,
        237
      ],
      "accounts": [
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "mint",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "programData",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  75,
                  26,
                  151,
                  3,
                  94,
                  224,
                  4,
                  213,
                  42,
                  225,
                  6,
                  126,
                  100,
                  169,
                  35,
                  160,
                  78,
                  118,
                  136,
                  21,
                  244,
                  10,
                  153,
                  245,
                  211,
                  105,
                  39,
                  89,
                  127,
                  20,
                  12,
                  90
                ]
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                2,
                168,
                246,
                145,
                78,
                136,
                161,
                176,
                226,
                16,
                21,
                62,
                247,
                99,
                174,
                43,
                0,
                194,
                185,
                61,
                22,
                193,
                36,
                210,
                192,
                83,
                122,
                16,
                4,
                128,
                0,
                0
              ]
            }
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "initializeArgs"
            }
          }
        }
      ]
    },
    {
      "name": "proposeAdmin",
      "discriminator": [
        121,
        214,
        199,
        212,
        87,
        39,
        117,
        234
      ],
      "accounts": [
        {
          "name": "admin",
          "signer": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "newAdmin",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "setPaused",
      "discriminator": [
        91,
        60,
        125,
        192,
        176,
        225,
        166,
        218
      ],
      "accounts": [
        {
          "name": "admin",
          "signer": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "round",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  111,
                  117,
                  110,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "round.id",
                "account": "round"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "paused",
          "type": "bool"
        }
      ]
    },
    {
      "name": "withdrawProceeds",
      "discriminator": [
        124,
        68,
        215,
        12,
        201,
        136,
        54,
        72
      ],
      "accounts": [
        {
          "name": "admin",
          "writable": true,
          "signer": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "round",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  111,
                  117,
                  110,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "round.id",
                "account": "round"
              }
            ]
          }
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "allocation",
      "discriminator": [
        147,
        154,
        3,
        177,
        155,
        25,
        131,
        176
      ]
    },
    {
      "name": "config",
      "discriminator": [
        155,
        12,
        170,
        224,
        30,
        250,
        204,
        130
      ]
    },
    {
      "name": "round",
      "discriminator": [
        87,
        127,
        165,
        51,
        73,
        78,
        116,
        174
      ]
    }
  ],
  "events": [
    {
      "name": "claimed",
      "discriminator": [
        217,
        192,
        123,
        72,
        108,
        150,
        248,
        33
      ]
    },
    {
      "name": "purchased",
      "discriminator": [
        20,
        112,
        33,
        232,
        177,
        248,
        215,
        233
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "notUpgradeAuthority",
      "msg": "Signer is not the program upgrade authority"
    },
    {
      "code": 6001,
      "name": "unauthorized",
      "msg": "Signer is not the sale admin"
    },
    {
      "code": 6002,
      "name": "notPendingAdmin",
      "msg": "Signer is not the pending admin"
    },
    {
      "code": 6003,
      "name": "invalidDecimals",
      "msg": "Decimals must be at most 9"
    },
    {
      "code": 6004,
      "name": "invalidSupplyCap",
      "msg": "Supply cap must be greater than zero"
    },
    {
      "code": 6005,
      "name": "invalidMetadata",
      "msg": "Name, symbol or URI is empty or too long"
    },
    {
      "code": 6006,
      "name": "invalidSchedule",
      "msg": "Round must end after it starts and in the future"
    },
    {
      "code": 6007,
      "name": "invalidPricing",
      "msg": "Price and hard cap must be greater than zero"
    },
    {
      "code": 6008,
      "name": "invalidVesting",
      "msg": "Cliff must be between zero and the vesting period, which is at most ten years"
    },
    {
      "code": 6009,
      "name": "supplyCapExceeded",
      "msg": "Round caps would exceed the supply cap"
    },
    {
      "code": 6010,
      "name": "roundPaused",
      "msg": "Round is paused"
    },
    {
      "code": 6011,
      "name": "roundNotStarted",
      "msg": "Round has not started"
    },
    {
      "code": 6012,
      "name": "roundEnded",
      "msg": "Round has ended"
    },
    {
      "code": 6013,
      "name": "roundNotEnded",
      "msg": "Round has not ended"
    },
    {
      "code": 6014,
      "name": "zeroAmount",
      "msg": "Amount must be greater than zero"
    },
    {
      "code": 6015,
      "name": "proofTooLong",
      "msg": "Merkle proof is longer than the maximum depth"
    },
    {
      "code": 6016,
      "name": "invalidProof",
      "msg": "Wallet and allocation are not in the allowlist"
    },
    {
      "code": 6017,
      "name": "allocationExceeded",
      "msg": "Purchase exceeds the wallet allocation"
    },
    {
      "code": 6018,
      "name": "hardCapExceeded",
      "msg": "Purchase exceeds the round hard cap"
    },
    {
      "code": 6019,
      "name": "nothingToClaim",
      "msg": "Nothing has vested since the last claim"
    },
    {
      "code": 6020,
      "name": "noProceeds",
      "msg": "Round has no proceeds to withdraw"
    },
    {
      "code": 6021,
      "name": "mathOverflow",
      "msg": "Arithmetic overflow"
    }
  ],
  "types": [
    {
      "name": "allocation",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "round",
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "maxAmount",
            "type": "u64"
          },
          {
            "name": "purchased",
            "type": "u64"
          },
          {
            "name": "claimed",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "claimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "round",
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "config",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "pendingAdmin",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "decimals",
            "type": "u8"
          },
          {
            "name": "supplyCap",
            "docs": [
              "Sum of all round hard caps can never exceed this."
            ],
            "type": "u64"
          },
          {
            "name": "allocated",
            "type": "u64"
          },
          {
            "name": "roundCount",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "initializeArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "decimals",
            "type": "u8"
          },
          {
            "name": "supplyCap",
            "type": "u64"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "symbol",
            "type": "string"
          },
          {
            "name": "uri",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "purchased",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "round",
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "cost",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "round",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "id",
            "type": "u64"
          },
          {
            "name": "startTs",
            "type": "i64"
          },
          {
            "name": "endTs",
            "type": "i64"
          },
          {
            "name": "price",
            "docs": [
              "Lamports per whole token (10^decimals base units)."
            ],
            "type": "u64"
          },
          {
            "name": "hardCap",
            "type": "u64"
          },
          {
            "name": "sold",
            "type": "u64"
          },
          {
            "name": "proceeds",
            "docs": [
              "Lamports held by this account for the admin."
            ],
            "type": "u64"
          },
          {
            "name": "merkleRoot",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "cliff",
            "docs": [
              "Seconds after `end_ts` before anything unlocks."
            ],
            "type": "i64"
          },
          {
            "name": "vesting",
            "docs": [
              "Seconds after `end_ts` until everything unlocks."
            ],
            "type": "i64"
          },
          {
            "name": "paused",
            "type": "bool"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "roundArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "startTs",
            "type": "i64"
          },
          {
            "name": "endTs",
            "type": "i64"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "hardCap",
            "type": "u64"
          },
          {
            "name": "merkleRoot",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "cliff",
            "type": "i64"
          },
          {
            "name": "vesting",
            "type": "i64"
          }
        ]
      }
    }
  ],
  "constants": [
    {
      "name": "allocationSeed",
      "type": "bytes",
      "value": "[97, 108, 108, 111, 99, 97, 116, 105, 111, 110]"
    },
    {
      "name": "configSeed",
      "type": "bytes",
      "value": "[99, 111, 110, 102, 105, 103]"
    },
    {
      "name": "mintSeed",
      "type": "bytes",
      "value": "[109, 105, 110, 116]"
    },
    {
      "name": "roundSeed",
      "type": "bytes",
      "value": "[114, 111, 117, 110, 100]"
    }
  ]
};
