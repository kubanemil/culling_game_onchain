pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("BwDuywJ5NxriMUEJ7EMKFth7joH3W2snbMfQtKp4nwrf");

#[program]
pub mod asset {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.init(ctx.bumps)
    }

    pub fn buy_token(ctx: Context<BuyToken>, amount: u64) -> Result<()> {
        ctx.accounts.buy_token(amount)
    }

    pub fn buy_card(ctx: Context<BuyCard>, card_id: u8) -> Result<()> {
        ctx.accounts.buy_card(card_id)
    }

    pub fn create_metadata(
        ctx: Context<SetMetadata>,
        card_id: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.create_metadata(card_id, name, symbol, uri)
    }

    pub fn update_metadata(
        ctx: Context<SetMetadata>,
        card_id: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.update_metadata(card_id, name, symbol, uri)
    }
}
