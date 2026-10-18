use anchor_lang::prelude::*;

use crate::error::SaleError;

/// Lamports owed for `amount` base units at `price` lamports per whole token.
/// Rounds up so a dust purchase can never be free.
pub fn purchase_cost(amount: u64, price: u64, decimals: u8) -> Result<u64> {
    let unit = 10u128
        .checked_pow(u32::from(decimals))
        .ok_or(SaleError::MathOverflow)?;
    let cost = u128::from(amount)
        .checked_mul(u128::from(price))
        .ok_or(SaleError::MathOverflow)?
        .div_ceil(unit);
    u64::try_from(cost).map_err(|_| SaleError::MathOverflow.into())
}

/// Tokens unlocked at `now` from `total`, vesting linearly from `start` over `duration`
/// seconds, with nothing unlocked before `start + cliff`.
pub fn vested_amount(total: u64, start: i64, cliff: i64, duration: i64, now: i64) -> Result<u64> {
    let cliff_end = start.checked_add(cliff).ok_or(SaleError::MathOverflow)?;
    if now < cliff_end {
        return Ok(0);
    }
    let elapsed = now.checked_sub(start).ok_or(SaleError::MathOverflow)?;
    if elapsed >= duration {
        return Ok(total);
    }
    let elapsed = u64::try_from(elapsed).map_err(|_| SaleError::MathOverflow)?;
    let duration = u64::try_from(duration).map_err(|_| SaleError::MathOverflow)?;
    let vested = u128::from(total)
        .checked_mul(u128::from(elapsed))
        .ok_or(SaleError::MathOverflow)?
        .checked_div(u128::from(duration))
        .ok_or(SaleError::MathOverflow)?;
    u64::try_from(vested).map_err(|_| SaleError::MathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400;

    #[test]
    fn cost_scales_with_decimals() {
        assert_eq!(purchase_cost(1_000_000, 2_000, 6).unwrap(), 2_000);
        assert_eq!(purchase_cost(500_000, 2_000, 6).unwrap(), 1_000);
        assert_eq!(purchase_cost(3, 7, 0).unwrap(), 21);
    }

    #[test]
    fn cost_rounds_up_so_dust_is_never_free() {
        assert_eq!(purchase_cost(1, 1, 9).unwrap(), 1);
        assert_eq!(purchase_cost(1_500_000, 3, 6).unwrap(), 5);
    }

    #[test]
    fn cost_overflow_is_an_error() {
        assert!(purchase_cost(u64::MAX, u64::MAX, 0).is_err());
        assert_eq!(purchase_cost(u64::MAX, 1, 0).unwrap(), u64::MAX);
    }

    #[test]
    fn nothing_vests_before_the_cliff() {
        assert_eq!(
            vested_amount(1_000, 0, 30 * DAY, 100 * DAY, 30 * DAY - 1).unwrap(),
            0
        );
        assert_eq!(vested_amount(1_000, 0, 30 * DAY, 100 * DAY, -5).unwrap(), 0);
    }

    #[test]
    fn cliff_releases_the_linear_share_so_far() {
        assert_eq!(
            vested_amount(1_000, 0, 30 * DAY, 100 * DAY, 30 * DAY).unwrap(),
            300
        );
    }

    #[test]
    fn vesting_is_linear_and_rounds_down() {
        assert_eq!(
            vested_amount(1_000, 0, 0, 100 * DAY, 50 * DAY).unwrap(),
            500
        );
        assert_eq!(vested_amount(10, 0, 0, 3, 1).unwrap(), 3);
        assert_eq!(vested_amount(10, 0, 0, 3, 2).unwrap(), 6);
    }

    #[test]
    fn everything_vests_at_the_end() {
        assert_eq!(
            vested_amount(1_000, 0, 0, 100 * DAY, 100 * DAY).unwrap(),
            1_000
        );
        assert_eq!(
            vested_amount(1_000, 0, 0, 100 * DAY, i64::MAX).unwrap(),
            1_000
        );
    }

    #[test]
    fn zero_duration_unlocks_at_start() {
        assert_eq!(vested_amount(1_000, 50, 0, 0, 49).unwrap(), 0);
        assert_eq!(vested_amount(1_000, 50, 0, 0, 50).unwrap(), 1_000);
    }

    #[test]
    fn vesting_never_decreases_and_never_exceeds_total() {
        let total = u64::MAX;
        let mut last = 0;
        for now in (0..=120).map(|d| d * DAY) {
            let vested = vested_amount(total, 0, 10 * DAY, 100 * DAY, now).unwrap();
            assert!(vested >= last && vested <= total);
            last = vested;
        }
        assert_eq!(last, total);
    }
}
