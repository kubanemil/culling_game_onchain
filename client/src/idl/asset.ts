/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/asset.json`.
 */
export type Asset = {
  "address": "4DZmGYrQ2ZuzAz2ZeJ9ucFRyrZG9h5bcmwsZ7c2dyLAa",
  "metadata": {
    "name": "asset",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Card mints with on-chain stats, metadata gated by governance"
  },
  "instructions": [
    {
      "name": "buyCard",
      "discriminator": [
        113,
        142,
        149,
        246,
        22,
        115,
        156,
        154
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
          "name": "card",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "cardMint"
              }
            ]
          }
        },
        {
          "name": "cardMint",
          "writable": true
        },
        {
          "name": "tokenMint",
          "writable": true,
          "address": "B6zwgemZjj91wNPXQ7w6j5dWdQ3b8QT8caLD73Wp7XTf"
        },
        {
          "name": "buyerTokens",
          "writable": true
        },
        {
          "name": "buyerCards",
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
                "path": "cardMint"
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
      "name": "createCard",
      "discriminator": [
        29,
        242,
        127,
        8,
        241,
        109,
        219,
        100
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
          "name": "mint",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  97,
                  114,
                  100,
                  95,
                  109,
                  105,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "config.cardCount",
                "account": "config"
              }
            ]
          }
        },
        {
          "name": "card",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
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
              "name": "createCardArgs"
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
          "name": "programData",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  47,
                  203,
                  19,
                  117,
                  13,
                  93,
                  61,
                  10,
                  222,
                  90,
                  196,
                  160,
                  94,
                  240,
                  211,
                  70,
                  83,
                  0,
                  55,
                  131,
                  144,
                  217,
                  242,
                  92,
                  178,
                  111,
                  5,
                  51,
                  103,
                  67,
                  86,
                  55
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
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "updateCardMetadata",
      "discriminator": [
        170,
        157,
        17,
        138,
        58,
        199,
        72,
        170
      ],
      "accounts": [
        {
          "name": "governanceAuthority",
          "docs": [
            "Only the governance program can produce this signature, after a proposal passes."
          ],
          "signer": true,
          "address": "J3LcDf59bVg8QmSKvD252CKqYgCATAbrj8RNbEx2sz4D"
        },
        {
          "name": "payer",
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
          "name": "card",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  97,
                  114,
                  100
                ]
              },
              {
                "kind": "account",
                "path": "cardMint"
              }
            ]
          }
        },
        {
          "name": "cardMint",
          "writable": true
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
          "name": "field",
          "type": {
            "defined": {
              "name": "cardField"
            }
          }
        },
        {
          "name": "value",
          "type": "string"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "card",
      "discriminator": [
        166,
        250,
        46,
        230,
        152,
        63,
        140,
        182
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
    }
  ],
  "events": [
    {
      "name": "cardMetadataUpdated",
      "discriminator": [
        60,
        121,
        75,
        10,
        132,
        149,
        59,
        142
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
      "msg": "Signer is not the asset admin"
    },
    {
      "code": 6002,
      "name": "invalidStats",
      "msg": "Attack and health must be 1-100, defense 0-100"
    },
    {
      "code": 6003,
      "name": "invalidPrice",
      "msg": "Card price must be greater than zero"
    },
    {
      "code": 6004,
      "name": "invalidMetadata",
      "msg": "Name, symbol or URI is empty or too long"
    },
    {
      "code": 6005,
      "name": "mathOverflow",
      "msg": "Arithmetic overflow"
    }
  ],
  "types": [
    {
      "name": "card",
      "docs": [
        "One per card type. The mint holds name, symbol and URI; this holds what the game reads."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "id",
            "type": "u16"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "stats",
            "type": {
              "defined": {
                "name": "stats"
              }
            }
          },
          {
            "name": "price",
            "docs": [
              "Game tokens (base units) burned to mint one card."
            ],
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
      "name": "cardField",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "name"
          },
          {
            "name": "symbol"
          },
          {
            "name": "uri"
          }
        ]
      }
    },
    {
      "name": "cardMetadataUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "card",
            "type": "pubkey"
          },
          {
            "name": "field",
            "type": {
              "defined": {
                "name": "cardField"
              }
            }
          },
          {
            "name": "value",
            "type": "string"
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
            "name": "cardCount",
            "type": "u16"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "createCardArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stats",
            "type": {
              "defined": {
                "name": "stats"
              }
            }
          },
          {
            "name": "price",
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
      "name": "stats",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "attack",
            "type": "u16"
          },
          {
            "name": "defense",
            "type": "u16"
          },
          {
            "name": "health",
            "type": "u16"
          }
        ]
      }
    }
  ],
  "constants": [
    {
      "name": "cardMintSeed",
      "type": "bytes",
      "value": "[99, 97, 114, 100, 95, 109, 105, 110, 116]"
    },
    {
      "name": "cardSeed",
      "type": "bytes",
      "value": "[99, 97, 114, 100]"
    },
    {
      "name": "configSeed",
      "type": "bytes",
      "value": "[99, 111, 110, 102, 105, 103]"
    }
  ]
};
