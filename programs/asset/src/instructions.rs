pub mod buy_card;
pub mod create_card;
pub mod initialize;
pub mod update_card_metadata;

pub use buy_card::*;
pub use create_card::*;
pub use initialize::*;
pub use update_card_metadata::*;

use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

/// Token-2022 resizes the mint when metadata grows but leaves rent to the caller.
pub(crate) fn fund_rent<'info>(
    payer: AccountInfo<'info>,
    account: AccountInfo<'info>,
    system_program: Pubkey,
) -> Result<()> {
    let missing = Rent::get()?
        .minimum_balance(account.data_len())
        .saturating_sub(account.lamports());
    if missing > 0 {
        transfer(
            CpiContext::new(
                system_program,
                Transfer {
                    from: payer,
                    to: account,
                },
            ),
            missing,
        )?;
    }
    Ok(())
}
