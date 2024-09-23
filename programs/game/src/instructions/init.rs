use crate::state::Config;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: auth - wallet in backend that approves game's result.
    pub auth: UncheckedAccount<'info>,

    #[account(init, payer=signer, space=Config::INIT_SPACE, seeds=[b"config"], bump)]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitConfig<'info> {
    pub fn init(&mut self, bumps: InitConfigBumps) -> Result<()> {
        self.config.set_inner(Config {
            auth: self.auth.key(),
            owner: self.signer.key(),
            bump: bumps.config,
        });
        Ok(())
    }
}
