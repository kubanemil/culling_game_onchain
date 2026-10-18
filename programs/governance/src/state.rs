use anchor_lang::prelude::*;
use asset::{CardField, MAX_URI_LEN};

use crate::error::GovernanceError;

/// Fixed at init; no instruction changes it.
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub mint: Pubkey,
    /// Minimum yes + no votes, in token base units.
    pub quorum: u64,
    pub threshold_bps: u16,
    pub voting_period: i64,
    pub proposal_count: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalStatus {
    Voting,
    Executed,
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub card: Pubkey,
    pub card_mint: Pubkey,
    pub field: CardField,
    #[max_len(MAX_URI_LEN)]
    pub value: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub yes: u64,
    pub no: u64,
    pub status: ProposalStatus,
    pub bump: u8,
}

impl Proposal {
    pub fn check_passed(&self, config: &Config) -> Result<()> {
        let cast = self
            .yes
            .checked_add(self.no)
            .ok_or(GovernanceError::MathOverflow)?;
        require!(cast >= config.quorum, GovernanceError::QuorumNotReached);
        let approval = u128::from(self.yes)
            .checked_mul(10_000)
            .ok_or(GovernanceError::MathOverflow)?;
        let required = u128::from(cast)
            .checked_mul(u128::from(config.threshold_bps))
            .ok_or(GovernanceError::MathOverflow)?;
        require!(approval > required, GovernanceError::ThresholdNotReached);
        Ok(())
    }
}

/// Voting escrow: the ATA of this PDA holds the voting power.
#[account]
#[derive(InitSpace)]
pub struct Voter {
    pub owner: Pubkey,
    /// End of the last proposal voted on, so the same tokens cannot vote from two wallets.
    pub locked_until: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct VoteRecord {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub approve: bool,
    pub weight: u64,
    pub bump: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(quorum: u64, threshold_bps: u16) -> Config {
        Config {
            mint: Pubkey::default(),
            quorum,
            threshold_bps,
            voting_period: 60,
            proposal_count: 0,
            bump: 0,
        }
    }

    fn proposal(yes: u64, no: u64) -> Proposal {
        Proposal {
            id: 0,
            proposer: Pubkey::default(),
            card: Pubkey::default(),
            card_mint: Pubkey::default(),
            field: CardField::Uri,
            value: String::new(),
            start_ts: 0,
            end_ts: 0,
            yes,
            no,
            status: ProposalStatus::Voting,
            bump: 0,
        }
    }

    #[test]
    fn majority_means_strictly_more_than_half() {
        let config = config(1, 5_000);
        assert!(proposal(51, 49).check_passed(&config).is_ok());
        assert!(proposal(50, 50).check_passed(&config).is_err());
    }

    #[test]
    fn quorum_counts_both_sides() {
        let config = config(100, 5_000);
        assert!(proposal(60, 39).check_passed(&config).is_err());
        assert!(proposal(60, 40).check_passed(&config).is_ok());
    }

    #[test]
    fn supermajority_threshold() {
        let config = config(1, 6_667);
        assert!(proposal(66, 34).check_passed(&config).is_err());
        assert!(proposal(67, 33).check_passed(&config).is_ok());
    }

    #[test]
    fn huge_tallies_do_not_overflow() {
        let config = config(1, 9_999);
        assert!(proposal(u64::MAX - 1, 0).check_passed(&config).is_ok());
        assert!(proposal(u64::MAX, 1).check_passed(&config).is_err());
    }
}
