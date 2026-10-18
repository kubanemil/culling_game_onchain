use anchor_lang::prelude::*;

use crate::state::CardField;

#[event]
pub struct CardMetadataUpdated {
    pub card: Pubkey,
    pub field: CardField,
    pub value: String,
}
