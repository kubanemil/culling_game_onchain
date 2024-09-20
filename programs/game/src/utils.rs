use anchor_lang::{
    prelude::*,
    solana_program::{program, system_instruction},
};

pub fn transfer_lamports<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    lamports: u64,
) -> Result<()> {
    let transfer_ix = system_instruction::transfer(from.key, to.key, lamports);
    program::invoke(&transfer_ix, &[from, to])?;
    Ok(())
}
