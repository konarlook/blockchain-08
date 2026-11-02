use anchor_lang::prelude::*;

use crate::{constants::LAMPORTS_PER_SOL, error::MinterError};

pub fn calc_fee_lamports(fee_usd: u64, price: u64) -> Result<u64> {
    require!(price > 0, MinterError::InvalidOraclePrice);

    let lamports = u128::from(fee_usd)
        .checked_mul(u128::from(LAMPORTS_PER_SOL))
        .ok_or(MinterError::MathOverflow)?
        .checked_div(u128::from(price))
        .ok_or(MinterError::MathOverflow)?;

    u64::try_from(lamports).map_err(|_| error!(MinterError::MathOverflow))
}

#[cfg(test)]
mod tests {
    use super::*;

    const USD: u64 = 1_000_000;

    #[test]
    fn one_usd_at_25_usd_per_sol_is_0_04_sol() {
        assert_eq!(calc_fee_lamports(USD, 25 * USD).unwrap(), 40_000_000);
    }

    #[test]
    fn ten_usd_at_20_usd_per_sol_is_half_sol() {
        assert_eq!(calc_fee_lamports(10 * USD, 20 * USD).unwrap(), 500_000_000);
    }

    #[test]
    fn non_integer_result_is_rounded_down() {
        assert_eq!(calc_fee_lamports(10 * USD, 30 * USD).unwrap(), 333_333_333);
    }

    #[test]
    fn zero_fee_is_zero_lamports() {
        assert_eq!(calc_fee_lamports(0, 25 * USD).unwrap(), 0);
    }

    #[test]
    fn zero_price_is_rejected() {
        assert_eq!(
            calc_fee_lamports(USD, 0).unwrap_err(),
            MinterError::InvalidOraclePrice.into()
        );
    }

    #[test]
    fn result_not_fitting_u64_is_rejected() {
        assert_eq!(
            calc_fee_lamports(u64::MAX, 1).unwrap_err(),
            MinterError::MathOverflow.into()
        );
    }
}
