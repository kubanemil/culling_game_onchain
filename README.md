# Culling Game

A card-duel economy on Solana, written in Anchor. A token sale sells the game token to an
allowlist, with per-wallet caps and vesting. Players burn tokens to mint cards, each card type
with attack, defense and health stored on-chain. Two players stake SOL on a duel, each commits
to a card in secret, and the program decides the winner from both cards and randomness that
neither player controls. Token holders vote to change card metadata, and that vote is the only
way the metadata can change. Four programs, no admin key anywhere near a duel result.

## How it fits together

```
                 upgrade authority: initializes each program once

 +-----------------------------+  pins the mint  +--------------------------------+
 | sale                        |<----------------| governance                     |
 |  Config      "config"       |                 |  Config      "config"          |
 |  token mint  "mint"         |                 |  Proposal    "proposal", id    |
 |  Round       "round", id    |                 |  Voter       "voter", wallet   |
 |  Allocation  "allocation",  |                 |  VoteRecord  "vote", proposal, |
 |              round, buyer   |                 |              wallet            |
 +-----------------------------+                 |  authority   "authority"       |
       |                                         +--------------------------------+
       | mints CULL on claim                                    |
       v                                                        | execute_proposal:
  player wallet --- burns CULL, gets a card ---+                | CPI signed by
                                               |                | the authority PDA
                                               v                v
                                  +--------------------------------+
                                  | asset                          |  CPI
                                  |  Config     "config"           |-----> Token-2022
                                  |  card mint  "card_mint", id    |  metadata lives
                                  |  Card       "card", mint       |  in each mint
                                  +--------------------------------+
                                               ^
                                               | reads Card stats at reveal
                                  +--------------------------------+
                                  | duel                           |
                                  |  Duel  "duel", creator, nonce  |
                                  |  escrow "escrow", duel, player |
                                  |  stakes and revealed cards     |
                                  |  wait until each side claims   |
                                  +--------------------------------+
```

The game token and every card are Token-2022 mints. Their name, symbol and URI sit in the mint
through the metadata pointer and token metadata extensions, so there is no Metaplex program to
deploy or ship as a binary. The game token's metadata has no update authority, so it can never
change.

## Accounts

| Program | Account | Seeds | What it holds |
| --- | --- | --- | --- |
| sale | Config | `"config"` | admin, pending admin, mint, decimals, supply cap, supply reserved by rounds, round count |
| sale | token mint | `"mint"` | the game token; the sale Config is its mint authority |
| sale | Round | `"round"`, id | start and end, price per whole token, hard cap, sold, proceeds in lamports, merkle root, cliff, vesting, paused, finalized |
| sale | Allocation | `"allocation"`, round, buyer | the cap from the allowlist, amount bought, amount claimed; closed on the last claim |
| asset | Config | `"config"` | admin, pending admin and card count; mint authority and metadata update authority of every card |
| asset | card mint | `"card_mint"`, id | Token-2022 mint, 0 decimals, metadata in the mint |
| asset | Card | `"card"`, mint | attack, defense, health and price in game tokens |
| duel | Duel | `"duel"`, creator, nonce | both stakes, both commitments, the reveals, the deadline and who has claimed; closed by the creator's claim or `cancel_duel` |
| duel | card escrow | `"escrow"`, duel, player | the card a player revealed, owned by the Duel until that player claims |
| governance | Config | `"config"` | pinned mint, quorum, threshold, voting period, proposal count; fixed after init |
| governance | authority | `"authority"` | no data; the one signer the asset program accepts for metadata changes |
| governance | Proposal | `"proposal"`, id | card mint, field, new value, voting window, yes and no tallies, status |
| governance | FieldVersion | `"field"`, card mint, field | the last proposal executed for that field of that card |
| governance | Voter | `"voter"`, wallet | the weight locked by votes and until when; its token account holds the deposited tokens |
| governance | VoteRecord | `"vote"`, proposal, wallet | the side and the weight of one vote |

Round, card and proposal ids come from a counter in each program's Config and are encoded
little-endian in the seeds. The duel nonce is picked by the creator.

## Security model

Each row is an attack and the test that tries it. Most rows come from an audit of the old
version of this repo and of `whitelist_sale`, which this repo absorbed. Integration tests are in
[`tests/src`](tests/src); the merkle and pricing tests sit next to the code they cover.

| Attack | How it is stopped | Test |
| --- | --- | --- |
| Change card metadata without a vote | The asset Config PDA is the Token-2022 update authority. The asset program only signs an update when the governance authority PDA signs the call, and governance only signs in `execute_proposal` | `attack_a_non_governance_signer_cannot_change_card_metadata`, `attack_the_admin_cannot_edit_metadata_through_token_2022_directly` |
| Vote twice | One VoteRecord per `["vote", proposal, wallet]`; a second vote fails to create it | `attack_a_voter_cannot_vote_twice` |
| Vote with a self-minted token | The mint is pinned in the governance Config; deposits and votes check it | `attack_a_self_minted_token_has_no_voting_power` |
| Vote, move the tokens to another wallet, vote again | The weight a voter used stays locked until the last proposal they voted on ends | `attack_tokens_stay_locked_until_the_vote_ends_so_they_cannot_vote_again` |
| Voting tokens locked forever, or deposits made after a vote locked with it | `withdraw` frees everything above the locked weight now, and the rest once the lock passes | `tokens_deposited_after_a_vote_stay_withdrawable` |
| Execute twice, or execute a failed vote | Status moves from Voting to Executed once; quorum and threshold are checked first | `a_passed_proposal_updates_card_metadata_exactly_once`, `attack_a_proposal_without_quorum_or_majority_cannot_execute` |
| Execute an old passed proposal to undo a newer one | A FieldVersion per card field records the last executed proposal id; an older id is rejected | `attack_an_old_passed_proposal_cannot_undo_a_newer_change` |
| Keep a passed proposal and execute it months later | A passed proposal can only run within 7 days of the vote ending | `attack_a_passed_proposal_expires_after_the_execution_window` |
| Point a passed proposal at another card | The proposal stores the card mint; execute checks it and derives the card from it | `attack_a_passed_proposal_cannot_be_pointed_at_another_card` |
| One proposal per user ever, URIs over 28 bytes | Proposals are keyed by a counter; `InitSpace` counts the string prefix and allows 200 bytes | `one_wallet_can_open_many_proposals`, `a_passed_proposal_updates_card_metadata_exactly_once` |
| Be the first to initialize a program | `initialize` requires the program's upgrade authority | `attack_only_the_upgrade_authority_can_initialize` (one per program) |
| Accept a duel twice and get paid twice | Accept only works while the duel is Open | `attack_accept_cannot_be_called_twice` |
| Duel yourself | Rejected at create and at accept | `attack_a_player_cannot_duel_themselves` |
| Pick the winner | Nobody does: the winner comes from both reveals. A player who does not reveal in time loses | `a_full_duel_pays_each_side_its_share_and_returns_the_cards`, `attack_a_non_revealing_player_forfeits_after_timeout` |
| Change your card after seeing the other side | The commitment binds the card, the secret, the duel and the player | `attack_a_wrong_secret_or_card_does_not_match_the_commitment` |
| Reveal a card you do not hold | Reveal moves one unit of the card from the player into escrow | `attack_a_player_cannot_reveal_a_card_they_no_longer_hold` |
| Back two reveals with one card, or borrow a card for one transaction | The card stays in escrow from reveal until its owner claims | `attack_one_card_cannot_back_two_reveals_at_once`, `attack_a_card_borrowed_for_one_transaction_cannot_be_handed_back_in_it` |
| Lose, then make your wallet refuse payments so the pot is stuck | Nothing is pushed to a player. Each side claims its own share and card, so a player can only block themselves | `attack_a_loser_cannot_lock_the_pot_by_reassigning_their_wallet` |
| Claim someone else's share, or claim twice | Only the two players can claim, once each | `attack_strangers_cannot_claim_and_players_cannot_claim_twice` |
| Dust duels whose payout cannot land in an empty wallet | Stakes start at 0.001 SOL, above the rent minimum of an empty account | `attack_a_dust_stake_is_rejected` |
| Stake stuck in a duel | The creator can cancel before accept. After accept, both sides can claim once both revealed or the hour is up | `the_creator_can_cancel_before_accept_and_gets_everything_back`, `if_nobody_reveals_both_stakes_are_refunded_after_timeout` |
| Every allowlisted buyer is an admin | One admin in the sale Config, moved only by propose then accept | `admin_transfer_takes_two_steps`, `attack_a_non_admin_cannot_create_or_pause_rounds` |
| Admin key that can never move | The asset admin moves by propose then accept, like the sale admin | `asset_admin_transfer_takes_two_steps` |
| Edit the game token's name or symbol | The token's metadata has no update authority | `attack_nobody_can_edit_the_game_token_metadata` |
| Unsold supply reserved forever | `finalize_round` returns a finished round's unsold cap to the supply cap, once | `finalizing_a_round_returns_its_unsold_supply`, `attack_a_round_cannot_be_finalized_early_or_twice` |
| Exceed the allocation by moving tokens out | Purchases are counted in the Allocation PDA, not read from a balance | `attack_a_buyer_cannot_exceed_allocation_by_moving_tokens` |
| Buy without being listed, or claim a bigger cap | Each buy proves `(wallet, cap)` against the round's merkle root | `attack_a_wallet_outside_the_allowlist_cannot_buy`, `attack_a_buyer_cannot_claim_a_bigger_allocation_than_listed` |
| Allowlist limited to 9 wallets | A merkle root covers any number of wallets | `every_member_verifies_for_any_tree_size` |
| Overflow the price or ignore decimals | u128 checked math, price per whole token, cost rounded up | `buy_charges_a_decimals_correct_price_into_the_round`, `cost_rounds_up_so_dust_is_never_free` |
| Buy outside the window, past the cap or while paused | Rounds have start, end, hard cap and pause; all caps together stay under the supply cap | `buying_is_blocked_while_paused_and_outside_the_window`, `attack_the_hard_cap_holds_across_buyers`, `round_caps_can_never_exceed_the_supply_cap` |
| Claim twice or claim someone else's tokens | Claims pay only what vested since the last claim, from the caller's own Allocation | `attack_a_buyer_cannot_claim_the_same_tokens_twice`, `attack_a_buyer_cannot_claim_someone_elses_allocation` |
| Withdraw sale proceeds early, or as someone else | Only the admin, only after the round ends, only once | `admin_withdraws_proceeds_once_and_only_after_the_round_ends`, `attack_a_non_admin_cannot_withdraw_proceeds` |
| Pay for a card with a self-minted token | The token mint is checked against the sale's mint address | `attack_paying_for_a_card_with_a_self_minted_token_fails` |

What is still trusted: the upgrade authority can replace any of the four programs. Handing it
to a multisig, or making the programs immutable, closes that; see the roadmap.

## Randomness

Each player commits to `sha256(duel, player, card mint, secret)`: the creator when creating
the duel, the opponent when accepting it. Once accepted, both have one hour to reveal their
card and secret. The random seed is `sha256(duel, creator secret, opponent secret)`. The creator
commits before anything is known, and the opponent sees only a hash, so neither can steer the
seed. Validators cannot either, because nothing comes from slot hashes or the clock.

The cards then fight. Both strike at once each round, dealing `attack + roll(0-7) - defense`,
at least 1, with the rolls taken from the seed. The fight stops when a card drops to zero or
after 16 rounds, and the card with more health left wins. Equal health is a draw and both stakes
go back.

The weak spot of commit-reveal is the second player to reveal: they learn the result first and
can refuse to reveal. Here that does not help them. A player who does not reveal within the hour
forfeits the whole pot, so walking away from a loss is still a loss, and walking away from a
draw turns it into one. The honest player only loses time, at most an hour. If neither reveals,
both get their stake back.

Revealing moves the card into an escrow owned by the duel, and it comes back when its owner
claims. Escrowing at commit time would show the opponent which card is coming. Both sides may
play the same card type, each with its own unit; refusing that would make the second revealer
forfeit over a choice they could not see.

The secret has to be 32 random bytes. There are only a few card types, so a guessable secret
would let the opponent recover the creator's card from the commitment before accepting. The
client generates it with a CSPRNG.

## Layout

```
programs/sale         token sale: rounds, merkle allowlist, vesting, proceeds
programs/asset        card mints, stats, governance-gated metadata
programs/duel         commit-reveal duels and the fight
programs/governance   proposals, escrowed voting, execute-once CPI into asset
crates/common         metadata limits and the rent top-up shared by sale and asset
tests/                LiteSVM integration tests, one binary
client/               TypeScript client built on the IDLs, and the walkthrough
do                    task runner
```

Pure logic has unit tests next to it: vesting and pricing in `programs/sale/src/math.rs`, merkle
proofs in `programs/sale/src/merkle.rs`, the fight in `programs/duel/src/fight.rs`, and quorum
and threshold in `programs/governance/src/state.rs`.

## Development

You need Rust 1.99.1 (pinned in `rust-toolchain.toml`), the Agave Solana CLI 4.3.2 and the Anchor
CLI 1.2.1. The client needs Node 24.

```
./do build   build the programs, IDLs and TS types
./do test    build, then run the unit tests and the LiteSVM integration tests
./do check   rustfmt --check and clippy with warnings as errors
./do fmt     format the Rust code
./do idl     build and copy the IDLs and TS types into client/src/idl
./do keys    give a fresh clone its own program IDs, to deploy your own copy
```

The integration tests load `target/deploy/*.so` into LiteSVM, so they run in-process in a few
seconds with no validator. CI runs `check` and `test`, and type-checks the client, on every
pull request.

Program keypairs stay in `target/deploy` and are never committed. After a fresh clone,
`./do keys` writes new IDs into the sources and `Anchor.toml`, including the copy of the
governance ID that the asset program uses to recognize the governance signer. A test fails if
that copy drifts.

## Walkthrough

`client/scripts/walkthrough.ts` runs the whole loop against a cluster where the programs are
deployed and your wallet is their upgrade authority: create a sale round, buy, claim, buy
cards, duel, deposit, propose, vote and execute. On a local validator:

```
solana-test-validator --reset            # in another terminal
anchor deploy
cd client && npm ci
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 ANCHOR_WALLET=~/.config/solana/id.json npm run walkthrough
```

For devnet, deploy with `anchor deploy --provider.cluster devnet` and point
`ANCHOR_PROVIDER_URL` at `https://api.devnet.solana.com`. Rent for the four programs is about
9 SOL. A local run prints:

```
== Sale: allowlisted round, buy, claim after it ends
round 0: both bought 50 CULL, waiting for it to end
claimed; admin withdrew the proceeds; round finalized

== Cards: Alice buys Brute, Bob buys Guard (10 CULL each, burned)

== Duel: commit, accept, reveal (cards go into escrow), claim
duel 52vdcWLv7nRKUQ8iDdbUb5MxBwuHct6wGt84Ke4jpbfJ: Bob wins; both cards are back

== Governance: deposit, propose a new URI for Brute, vote, execute
proposal 0: 80 CULL yes, voting ends at 1792312788
executed; Brute's URI is now https://example.com/cards/brute-v2.json
```

## Decisions

| Choice | Not chosen | Why |
| --- | --- | --- |
| Token-2022 for the game token and the cards | SPL Token with Metaplex metadata | Metadata lives in the mint, so there is no external program to deploy or commit as a binary, and one token program covers everything. The cost is `transfer_checked` everywhere and thinner wallet support. |
| Merkle root per round plus an Allocation PDA | A list of wallets in an account | One 32-byte root covers any number of wallets, and the cap is counted per wallet in the program instead of read from a balance. |
| Mint on claim | Mint at purchase into an escrow | No vault per buyer, and supply only grows as tokens unlock. Unsold tokens never exist. |
| Vesting from the round's end | From each purchase | Everyone in a round unlocks on the same schedule, stored once on the Round. |
| Round the price up | Round down | A purchase can never cost zero lamports. |
| Upgrade authority initializes | First caller becomes admin | Nobody can front-run the deploy and take the admin seat. |
| Card payments are burned | A treasury account | There is nothing for anyone to drain and no admin power over it. |
| Governance signer as a constant in asset | Stored in asset's config at init | Not even the deployer can point it at their own key. |
| Commit-reveal with forfeit | VRF, or slot hashes | No oracle, so it runs on localnet and in LiteSVM. Slot hashes can be seen and nudged by the leader. VRF is on the roadmap. |
| SOL stakes | Game token stakes | Fewer accounts per instruction. The token already pays for cards and votes. |
| Each player claims their own payout | One call that pays both wallets | A player controls their own wallet and can make it refuse payments. Pulling means each side can only block itself. |
| Card stats copied into the duel at reveal | Read again at claim | Claims need no Card accounts, and the result is fixed once both have revealed. |
| Lock the largest weight voted with, until the last voted proposal ends | Balance snapshots, or a lock per vote | Token-2022 has no snapshots, and one weight and one time per voter is enough to stop one balance voting through two wallets. It can lock a little more than needed when a voter tops up between overlapping votes, never less. |
| Newer proposals win per card field | Last executed wins | Votes on the same field can overlap, and running an older result after a newer one would undo the newer decision. |
| LiteSVM tests in Rust | `solana-test-validator` with mocha | In-process and fast, and the tests use the programs' own types and error enums. |

## Roadmap

Milestone 2:

- Switchboard VRF as the duel's randomness source, so a duel resolves without both players
  coming back to reveal.
- Elo ranking: a rating account per player, updated when a duel resolves.
- A Squads multisig as the admin of the sale and asset programs and as upgrade authority of
  all four.
- A dashboard backed by an indexer. The programs already emit `Purchased`, `Claimed`,
  `CardMetadataUpdated`, `DuelSettled`, `VoteCast` and `ProposalExecuted`, which chainpulse's
  planned Solana source could index from the IDLs.

## License

MIT
