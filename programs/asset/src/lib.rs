pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4DZmGYrQ2ZuzAz2ZeJ9ucFRyrZG9h5bcmwsZ7c2dyLAa");

#[program]
pub mod asset {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handle_initialize(ctx)
    }

    pub fn propose_admin(ctx: Context<ProposeAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::transfer_admin::handle_propose_admin(ctx, new_admin)
    }

    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        instructions::transfer_admin::handle_accept_admin(ctx)
    }

    pub fn create_card(ctx: Context<CreateCard>, args: CreateCardArgs) -> Result<()> {
        instructions::create_card::handle_create_card(ctx, args)
    }

    pub fn buy_card(ctx: Context<BuyCard>) -> Result<()> {
        instructions::buy_card::handle_buy_card(ctx)
    }

    pub fn update_card_metadata(
        ctx: Context<UpdateCardMetadata>,
        field: CardField,
        value: String,
    ) -> Result<()> {
        instructions::update_card_metadata::handle_update_card_metadata(ctx, field, value)
    }
}
