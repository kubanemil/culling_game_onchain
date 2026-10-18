use ::asset::{error::AssetError, Card, CardField, Config, Stats, MAX_URI_LEN};
use ::sale::TOKEN_MINT;
use anchor_lang::{error::ErrorCode, prelude::Pubkey};
use anchor_spl::{
    token_2022::spl_token_2022,
    token_interface::spl_token_metadata_interface::{
        error::TokenMetadataError, instruction::update_field, state::Field,
    },
};
use solana_signer::Signer;

use crate::{
    harness::{ata, expect_error, expect_log, Env},
    ix,
    world::{card_args, World, BRUTE, CARD_PRICE, DECIMALS, TOKEN},
};

#[test]
fn create_card_stores_stats_and_token_2022_metadata() {
    let world = World::new();
    let mint = world.cards[0];
    let card: Card = world.env.account(&ix::asset::card(&mint));
    assert_eq!(card.id, 0);
    assert_eq!(card.mint, mint);
    assert_eq!(card.stats, BRUTE);
    assert_eq!(card.price, CARD_PRICE);

    let metadata = world.env.metadata(&mint);
    assert_eq!(metadata.name, "Brute");
    assert_eq!(metadata.uri, "https://example.com/cards/brute.json");
    assert_eq!(
        Option::<Pubkey>::from(metadata.update_authority),
        Some(ix::asset::config())
    );

    let config: Config = world.env.account(&ix::asset::config());
    assert_eq!(config.card_count, 3);
}

#[test]
fn attack_only_the_upgrade_authority_can_initialize_asset() {
    let mut env = Env::new();
    let stranger = env.user();
    let result = env.send(&[ix::asset::initialize(&stranger.pubkey())], &[&stranger]);
    expect_error(result, AssetError::NotUpgradeAuthority);
}

#[test]
fn attack_a_non_admin_cannot_create_cards() {
    let mut world = World::new();
    let stranger = world.env.user();
    let result = world.env.send(
        &[ix::asset::create_card(
            &stranger.pubkey(),
            3,
            card_args("Fake", BRUTE),
        )],
        &[&stranger],
    );
    expect_error(result, AssetError::Unauthorized);
}

#[test]
fn card_stats_and_metadata_are_validated() {
    let mut world = World::new();
    let admin = world.admin();
    let bad_stats = [
        Stats { attack: 0, ..BRUTE },
        Stats { health: 0, ..BRUTE },
        Stats {
            defense: 101,
            ..BRUTE
        },
    ];
    for stats in bad_stats {
        let result = world.env.send(
            &[ix::asset::create_card(
                &admin.pubkey(),
                3,
                card_args("Bad", stats),
            )],
            &[&admin],
        );
        expect_error(result, AssetError::InvalidStats);
    }

    let mut args = card_args("Bad", BRUTE);
    args.uri = "x".repeat(MAX_URI_LEN + 1);
    let result = world.env.send(
        &[ix::asset::create_card(&admin.pubkey(), 3, args)],
        &[&admin],
    );
    expect_error(result, AssetError::InvalidMetadata);
}

#[test]
fn buy_card_burns_the_price_and_mints_one_card() {
    let mut world = World::new();
    let player = world.funded_players(1, 25 * TOKEN).remove(0);
    let supply = world.env.mint_supply(&TOKEN_MINT);

    world.buy_card(&player, 1);
    world.buy_card(&player, 1);

    assert_eq!(world.tokens_of(&player.pubkey()), 5 * TOKEN);
    assert_eq!(world.env.mint_supply(&TOKEN_MINT), supply - 2 * CARD_PRICE);
    let cards = ata(&player.pubkey(), &world.cards[1]);
    assert_eq!(world.env.token_balance(&cards), 2);

    let result = world.env.send(
        &[ix::asset::buy_card(&player.pubkey(), &world.cards[1])],
        &[&player],
    );
    expect_log(result, "insufficient funds");
}

#[test]
fn attack_paying_for_a_card_with_a_self_minted_token_fails() {
    let mut world = World::new();
    let attacker = world.env.user();
    let fake = world.env.fake_mint(&attacker, DECIMALS);
    world
        .env
        .mint_to(&attacker, &fake, &attacker.pubkey(), 1_000 * TOKEN);

    let result = world.env.send(
        &[ix::asset::buy_card_with(
            &attacker.pubkey(),
            &world.cards[0],
            &fake,
        )],
        &[&attacker],
    );
    expect_error(result, ErrorCode::ConstraintAddress);
}

#[test]
fn attack_a_non_governance_signer_cannot_change_card_metadata() {
    let mut world = World::new();
    let admin = world.admin();
    let mint = world.cards[0];
    for signer in [admin, world.env.user()] {
        let result = world.env.send(
            &[ix::asset::update_card_metadata(
                &signer.pubkey(),
                &signer.pubkey(),
                &mint,
                CardField::Uri,
                "https://evil.example/brute.json",
            )],
            &[&signer],
        );
        expect_error(result, ErrorCode::ConstraintAddress);
    }
    assert_eq!(
        world.env.metadata(&mint).uri,
        "https://example.com/cards/brute.json"
    );
}

#[test]
fn attack_the_admin_cannot_edit_metadata_through_token_2022_directly() {
    let mut world = World::new();
    let admin = world.admin();
    let mint = world.cards[0];
    let result = world.env.send(
        &[update_field(
            &spl_token_2022::ID,
            &mint,
            &admin.pubkey(),
            Field::Uri,
            "https://evil.example/brute.json".into(),
        )],
        &[&admin],
    );
    let code = TokenMetadataError::IncorrectUpdateAuthority as u32;
    expect_log(result, &format!("custom program error: {code:#x}"));
    assert_eq!(
        world.env.metadata(&mint).uri,
        "https://example.com/cards/brute.json"
    );
}
