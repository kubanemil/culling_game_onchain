use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

pub const MAX_NAME_LEN: usize = 32;
pub const MAX_SYMBOL_LEN: usize = 10;
pub const MAX_URI_LEN: usize = 200;

pub fn fits(value: &str, max_len: usize) -> bool {
    !value.is_empty() && value.len() <= max_len
}

pub fn valid_metadata(name: &str, symbol: &str, uri: &str) -> bool {
    fits(name, MAX_NAME_LEN) && fits(symbol, MAX_SYMBOL_LEN) && fits(uri, MAX_URI_LEN)
}

/// Token-2022 resizes a mint when its metadata grows but leaves rent to the caller.
pub fn fund_rent<'info>(
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
