use anchor_lang::prelude::*;
use anchor_spl::token_interface::spl_token_metadata_interface::state::Field;

use crate::constants::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub card_count: u16,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub attack: u16,
    pub defense: u16,
    pub health: u16,
}

impl Stats {
    pub fn is_valid(&self) -> bool {
        (1..=MAX_STAT).contains(&self.attack)
            && self.defense <= MAX_STAT
            && (1..=MAX_STAT).contains(&self.health)
    }
}

#[account]
#[derive(InitSpace)]
pub struct Card {
    pub id: u16,
    pub mint: Pubkey,
    pub stats: Stats,
    /// Game tokens (base units) burned to mint one card.
    pub price: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardField {
    Name,
    Symbol,
    Uri,
}

impl CardField {
    pub fn max_len(self) -> usize {
        match self {
            CardField::Name => MAX_NAME_LEN,
            CardField::Symbol => MAX_SYMBOL_LEN,
            CardField::Uri => MAX_URI_LEN,
        }
    }

    pub fn accepts(self, value: &str) -> bool {
        !value.is_empty() && value.len() <= self.max_len()
    }
}

impl From<CardField> for Field {
    fn from(field: CardField) -> Self {
        match field {
            CardField::Name => Field::Name,
            CardField::Symbol => Field::Symbol,
            CardField::Uri => Field::Uri,
        }
    }
}
