# Culling Game

A card-duel game on Solana. Two players stake SOL, each commits to a card in secret, and the
program picks the winner from both cards and randomness neither player controls. Cards are bought
by burning a token sold in an allowlisted sale, and card metadata changes only when token holders
vote for it.

```
programs/sale         token sale: rounds, merkle allowlist, per-wallet caps, vesting
programs/asset        cards: Token-2022 mints with metadata in the mint, stats in a PDA
programs/duel         commit-reveal duels and the fight
programs/governance   escrowed voting that executes passed proposals into asset
tests/                LiteSVM integration tests, attacks included
client/               TypeScript client and an end-to-end walkthrough
```

## How a duel is decided

Each player commits to `sha256(duel, player, card mint, secret)`: the creator when opening the
duel, the opponent when accepting it. After accept, both have an hour to reveal their card and
secret, and the fight is seeded with `sha256(duel, creator secret, opponent secret)`. The creator
commits before seeing anything and the opponent only sees a hash, so neither can steer the seed.
Nothing comes from slot hashes or the clock, so the block leader can't steer it either.

Each round both cards strike at once for `attack + roll(0..7) - defense`, at least 1. After 16
rounds or a knockout, the card with more health left wins. A tie refunds both stakes.

The weak spot of commit-reveal is the second player to reveal: they see the result first and can
walk away. Here a player who doesn't reveal within the hour forfeits the whole pot, so walking
away from a loss is still a loss. If neither reveals, both get their stake back.

The secret has to be 32 random bytes. With only a few card types, a guessable secret would let
the opponent recover the creator's card from the commitment. The client uses a CSPRNG.

## Attacks

Each of these has an integration test that tries it and expects it to fail:

- editing card metadata without a passed vote, including the admin going to Token-2022 directly ([asset.rs](tests/src/asset.rs))
- voting twice, voting with a self-minted token, or moving tokens to a second wallet to vote again ([governance.rs](tests/src/governance.rs))
- running an old passed proposal to undo a newer one, or pointing it at another card ([governance.rs](tests/src/governance.rs))
- changing your card after seeing the opponent's, or revealing a card you no longer hold ([duel.rs](tests/src/duel.rs))
- backing two reveals with one card, or borrowing a card for a single transaction ([duel.rs](tests/src/duel.rs))
- losing, then making your wallet refuse payments so the pot gets stuck ([duel.rs](tests/src/duel.rs))
- buying past your allowlist cap, or claiming vested tokens twice ([sale.rs](tests/src/sale.rs))
- calling `initialize` before the deployer does, to take the admin seat (sale, asset and governance)

What stays trusted is the upgrade authority, which can replace any of the four programs.

## Build and play

You need Rust 1.99.1 (pinned in `rust-toolchain.toml`), the Agave Solana CLI 4.3.2 and Anchor
1.2.1. The client needs Node 24.

```
./do test    # anchor build, then unit tests and the LiteSVM integration tests
./do check   # rustfmt --check and clippy with warnings as errors
```

The integration tests load `target/deploy/*.so` into LiteSVM, so there is no validator to start.

`client/scripts/walkthrough.ts` plays the whole loop: a sale round, cards, a duel and a vote. A
fresh clone needs its own program IDs first:

```
./do keys && ./do idl
solana-test-validator --reset   # in another terminal
anchor deploy
cd client && npm ci
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 ANCHOR_WALLET=~/.config/solana/id.json npm run walkthrough
```

MIT licensed.
