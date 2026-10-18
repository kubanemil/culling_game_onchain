//! The battle is a pure function of both cards and the shared randomness.
//! Both cards strike at the same time each round, so neither side moves first.

use asset::Stats;

/// Two random bytes per round, one per player.
pub const ROUNDS: usize = 16;

/// Each strike adds a roll of 0..ROLL_SIDES. 256 is a multiple of 8, so the roll is unbiased.
pub const ROLL_SIDES: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    CreatorWins,
    OpponentWins,
    Draw,
}

/// Health can go below zero, so when both cards fall in the same round the smaller
/// overkill wins and only an exact tie is a draw.
pub fn fight(creator: &Stats, opponent: &Stats, seed: &[u8; 32]) -> Outcome {
    let mut creator_hp = i32::from(creator.health);
    let mut opponent_hp = i32::from(opponent.health);
    for [creator_roll, opponent_roll] in seed.as_chunks::<2>().0.iter().take(ROUNDS) {
        opponent_hp -= damage(creator.attack, *creator_roll, opponent.defense);
        creator_hp -= damage(opponent.attack, *opponent_roll, creator.defense);
        if creator_hp <= 0 || opponent_hp <= 0 {
            break;
        }
    }
    match creator_hp.cmp(&opponent_hp) {
        std::cmp::Ordering::Greater => Outcome::CreatorWins,
        std::cmp::Ordering::Less => Outcome::OpponentWins,
        std::cmp::Ordering::Equal => Outcome::Draw,
    }
}

/// At least 1, so a duel between two walls still ends on health. Stats are u16, so 16
/// rounds of this cannot overflow an i32.
fn damage(attack: u16, roll: u8, defense: u16) -> i32 {
    (i32::from(attack) + i32::from(roll % ROLL_SIDES) - i32::from(defense)).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(attack: u16, defense: u16, health: u16) -> Stats {
        Stats {
            attack,
            defense,
            health,
        }
    }

    fn seed(i: u32) -> [u8; 32] {
        solana_sha256_hasher::hash(&i.to_le_bytes()).to_bytes()
    }

    fn swapped(seed: &[u8; 32]) -> [u8; 32] {
        let mut out = *seed;
        for pair in out.as_chunks_mut::<2>().0 {
            pair.swap(0, 1);
        }
        out
    }

    #[test]
    fn swapping_seats_mirrors_the_outcome() {
        let a = stats(12, 4, 40);
        let b = stats(9, 7, 45);
        for i in 0..500 {
            let s = seed(i);
            let mirrored = match fight(&b, &a, &swapped(&s)) {
                Outcome::CreatorWins => Outcome::OpponentWins,
                Outcome::OpponentWins => Outcome::CreatorWins,
                Outcome::Draw => Outcome::Draw,
            };
            assert_eq!(fight(&a, &b, &s), mirrored);
        }
    }

    #[test]
    fn mirror_match_with_equal_rolls_is_a_draw() {
        let a = stats(10, 5, 30);
        assert_eq!(fight(&a, &a, &[3; 32]), Outcome::Draw);
    }

    #[test]
    fn a_much_stronger_card_always_wins() {
        let strong = stats(60, 50, 100);
        let weak = stats(5, 0, 20);
        for i in 0..500 {
            assert_eq!(fight(&strong, &weak, &seed(i)), Outcome::CreatorWins);
            assert_eq!(fight(&weak, &strong, &seed(i)), Outcome::OpponentWins);
        }
    }

    #[test]
    fn equal_cards_are_decided_by_the_rolls() {
        let card = stats(12, 4, 40);
        let outcomes: Vec<Outcome> = (0..1_000).map(|i| fight(&card, &card, &seed(i))).collect();
        let creator = outcomes
            .iter()
            .filter(|o| **o == Outcome::CreatorWins)
            .count();
        let opponent = outcomes
            .iter()
            .filter(|o| **o == Outcome::OpponentWins)
            .count();
        assert!(creator > 400 && opponent > 400, "{creator} vs {opponent}");
    }

    #[test]
    fn damage_is_never_zero() {
        assert_eq!(damage(1, 0, 100), 1);
        assert_eq!(damage(10, 7, 5), 12);
    }

    #[test]
    fn walls_that_cannot_hurt_each_other_end_on_health() {
        let tank = stats(1, 100, 100);
        let glass = stats(1, 100, 20);
        assert_eq!(fight(&tank, &glass, &[0; 32]), Outcome::CreatorWins);
    }
}
