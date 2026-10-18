/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/duel.json`.
 */
export type Duel = {
  "address": "HcyprDcax4Ffz6h9s1sA8X4Zh6yWwZuipw2qFvxG6CkE",
  "metadata": {
    "name": "duel",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Trustless 1v1 card duels with commit-reveal"
  },
  "instructions": [
    {
      "name": "acceptDuel",
      "discriminator": [
        80,
        52,
        90,
        135,
        172,
        221,
        175,
        102
      ],
      "accounts": [
        {
          "name": "opponent",
          "writable": true,
          "signer": true
        },
        {
          "name": "duel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  117,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "duel.creator",
                "account": "duel"
              },
              {
                "kind": "account",
                "path": "duel.nonce",
                "account": "duel"
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
          "name": "commitment",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    },
    {
      "name": "cancelDuel",
      "discriminator": [
        83,
        124,
        224,
        237,
        235,
        44,
        38,
        57
      ],
      "accounts": [
        {
          "name": "creator",
          "writable": true,
          "signer": true,
          "relations": [
            "duel"
          ]
        },
        {
          "name": "duel",
          "docs": [
            "Closing returns the stake and the rent in one step."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  117,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "creator"
              },
              {
                "kind": "account",
                "path": "duel.nonce",
                "account": "duel"
              }
            ]
          }
        }
      ],
      "args": []
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
          "name": "player",
          "writable": true,
          "signer": true
        },
        {
          "name": "duel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  117,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "duel.creator",
                "account": "duel"
              },
              {
                "kind": "account",
                "path": "duel.nonce",
                "account": "duel"
              }
            ]
          }
        },
        {
          "name": "cardMint",
          "docs": [
            "The next three are needed only by a player who revealed, to get the card back."
          ],
          "optional": true
        },
        {
          "name": "escrow",
          "writable": true,
          "optional": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "duel"
              },
              {
                "kind": "account",
                "path": "player"
              }
            ]
          }
        },
        {
          "name": "playerCards",
          "writable": true,
          "optional": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
        }
      ],
      "args": []
    },
    {
      "name": "createDuel",
      "discriminator": [
        49,
        28,
        93,
        11,
        75,
        242,
        69,
        165
      ],
      "accounts": [
        {
          "name": "creator",
          "writable": true,
          "signer": true
        },
        {
          "name": "duel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  117,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "creator"
              },
              {
                "kind": "arg",
                "path": "nonce"
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
          "name": "nonce",
          "type": "u64"
        },
        {
          "name": "stake",
          "type": "u64"
        },
        {
          "name": "commitment",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "opponent",
          "type": {
            "option": "pubkey"
          }
        }
      ]
    },
    {
      "name": "reveal",
      "discriminator": [
        9,
        35,
        59,
        190,
        167,
        249,
        76,
        115
      ],
      "accounts": [
        {
          "name": "player",
          "writable": true,
          "signer": true
        },
        {
          "name": "duel",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  117,
                  101,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "duel.creator",
                "account": "duel"
              },
              {
                "kind": "account",
                "path": "duel.nonce",
                "account": "duel"
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
                "path": "card.mint",
                "account": "card"
              }
            ],
            "program": {
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
          }
        },
        {
          "name": "cardMint"
        },
        {
          "name": "playerCards",
          "writable": true
        },
        {
          "name": "escrow",
          "docs": [
            "The card stays here until claim, so one card cannot back two reveals or be",
            "borrowed for a single transaction. Escrowing at commit would show the card early."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  115,
                  99,
                  114,
                  111,
                  119
                ]
              },
              {
                "kind": "account",
                "path": "duel"
              },
              {
                "kind": "account",
                "path": "player"
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
          "name": "secret",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "duel",
      "discriminator": [
        126,
        229,
        210,
        60,
        177,
        135,
        124,
        224
      ]
    }
  ],
  "events": [
    {
      "name": "duelSettled",
      "discriminator": [
        254,
        160,
        50,
        193,
        155,
        112,
        122,
        64
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "stakeTooSmall",
      "msg": "Stake is below the minimum of 0.001 SOL"
    },
    {
      "code": 6001,
      "name": "selfPlay",
      "msg": "A player cannot duel themselves"
    },
    {
      "code": 6002,
      "name": "notOpen",
      "msg": "Duel is not open"
    },
    {
      "code": 6003,
      "name": "notAccepted",
      "msg": "Duel was not accepted"
    },
    {
      "code": 6004,
      "name": "wrongOpponent",
      "msg": "This duel is reserved for another opponent"
    },
    {
      "code": 6005,
      "name": "notAPlayer",
      "msg": "Signer is not a player in this duel"
    },
    {
      "code": 6006,
      "name": "alreadyRevealed",
      "msg": "Player has already revealed"
    },
    {
      "code": 6007,
      "name": "revealWindowClosed",
      "msg": "Reveal window has closed"
    },
    {
      "code": 6008,
      "name": "revealWindowOpen",
      "msg": "Reveal window is still open"
    },
    {
      "code": 6009,
      "name": "commitmentMismatch",
      "msg": "Card and secret do not match the commitment"
    },
    {
      "code": 6010,
      "name": "cardNotOwned",
      "msg": "Player does not hold the revealed card"
    },
    {
      "code": 6011,
      "name": "alreadyClaimed",
      "msg": "Player has already claimed"
    },
    {
      "code": 6012,
      "name": "missingCardAccounts",
      "msg": "Pass the card mint, escrow and your card account to get your card back"
    },
    {
      "code": 6013,
      "name": "wrongCard",
      "msg": "Card mint does not match the revealed card"
    },
    {
      "code": 6014,
      "name": "mathOverflow",
      "msg": "Arithmetic overflow"
    }
  ],
  "types": [
    {
      "name": "card",
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
      "name": "duel",
      "docs": [
        "Holds both stakes until each player claims their share. Closed by the creator's claim."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "creator",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u64"
          },
          {
            "name": "opponent",
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "stake",
            "type": "u64"
          },
          {
            "name": "status",
            "type": {
              "defined": {
                "name": "duelStatus"
              }
            }
          },
          {
            "name": "creatorCommitment",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "opponentCommitment",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "revealDeadline",
            "type": "i64"
          },
          {
            "name": "creatorReveal",
            "type": {
              "option": {
                "defined": {
                  "name": "revealed"
                }
              }
            }
          },
          {
            "name": "opponentReveal",
            "type": {
              "option": {
                "defined": {
                  "name": "revealed"
                }
              }
            }
          },
          {
            "name": "creatorClaimed",
            "type": "bool"
          },
          {
            "name": "opponentClaimed",
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
      "name": "duelSettled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "duel",
            "type": "pubkey"
          },
          {
            "name": "creator",
            "type": "pubkey"
          },
          {
            "name": "opponent",
            "type": "pubkey"
          },
          {
            "name": "winner",
            "docs": [
              "`None` on a draw or when neither player revealed."
            ],
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "stake",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "duelStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "open"
          },
          {
            "name": "accepted"
          }
        ]
      }
    },
    {
      "name": "revealed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "card",
            "type": "pubkey"
          },
          {
            "name": "stats",
            "docs": [
              "Copied at reveal so the outcome needs no card accounts."
            ],
            "type": {
              "defined": {
                "name": "stats"
              }
            }
          },
          {
            "name": "secret",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
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
      "name": "duelSeed",
      "type": "bytes",
      "value": "[100, 117, 101, 108]"
    },
    {
      "name": "escrowSeed",
      "docs": [
        "A revealed card waits in this token account until its owner claims."
      ],
      "type": "bytes",
      "value": "[101, 115, 99, 114, 111, 119]"
    },
    {
      "name": "minStake",
      "docs": [
        "Keeps every payout above the rent minimum of an empty wallet."
      ],
      "type": "u64",
      "value": "1000000"
    },
    {
      "name": "revealWindow",
      "docs": [
        "Seconds to reveal, counted from accept."
      ],
      "type": "i64",
      "value": "3600"
    }
  ]
};
