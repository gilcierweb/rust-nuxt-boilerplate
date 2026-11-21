use std::fmt;
use std::ops::{Add, Sub};

use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    Brl,
    Usd,
    Euro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// The amount in the minor unit of the currency (cents).
    pub amount_minor: i64,
    pub currency: Currency,
}

impl Money {
    #[allow(dead_code)]
    pub fn new(amount_minor: i64, currency: Currency) -> Self {
        Self {
            amount_minor,
            currency,
        }
    }

    /// Converts to a decimal (1050 -> 10.50).
    pub fn to_decimal(self) -> BigDecimal {
        BigDecimal::from(self.amount_minor) / BigDecimal::from(100)
    }

    /// Builds from a decimal (10.50 -> 1050), rounding to the nearest cent.
    ///
    /// Returns `None` when the scaled value does not fit an `i64`: a library
    /// helper refuses the input instead of panicking on it.
    #[allow(dead_code)]
    pub fn from_decimal(value: BigDecimal, currency: Currency) -> Option<Self> {
        let scaled = value * BigDecimal::from(100);
        let amount_minor = scaled.with_scale(0).to_i64()?;
        Some(Self {
            amount_minor,
            currency,
        })
    }

    /// Applies a share in basis points (2000 = 20%), rounding toward zero.
    ///
    /// Returns `None` on overflow instead of wrapping around.
    #[allow(dead_code)]
    pub fn apply_basis_points(&self, basis_points: i64) -> Option<Self> {
        let amount_minor = self
            .amount_minor
            .checked_mul(basis_points)?
            .checked_div(10_000)?;
        Some(Self {
            amount_minor,
            currency: self.currency,
        })
    }

    /// Adds two amounts of the same currency, refusing the operation when the
    /// currencies differ or the sum overflows.
    #[allow(dead_code)]
    pub fn checked_add(self, other: Self) -> Option<Self> {
        if self.currency != other.currency {
            return None;
        }
        self.amount_minor
            .checked_add(other.amount_minor)
            .map(|amount_minor| Self {
                amount_minor,
                currency: self.currency,
            })
    }

    /// Subtracts two amounts of the same currency, refusing the operation when
    /// the currencies differ or the difference underflows.
    #[allow(dead_code)]
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        if self.currency != other.currency {
            return None;
        }
        self.amount_minor
            .checked_sub(other.amount_minor)
            .map(|amount_minor| Self {
                amount_minor,
                currency: self.currency,
            })
    }
}

impl Add for Money {
    type Output = Self;

    /// Sums amounts of the same currency.
    ///
    /// The operator cannot refuse, so a mismatched currency panics: the
    /// checked path is [`Money::checked_add`], which returns `None` instead.
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
            .expect("Money::add: the two amounts must share a currency and fit an i64")
    }
}

impl Sub for Money {
    type Output = Self;

    /// Subtracts amounts of the same currency, with the same panic contract
    /// as [`Add`]: the checked path is [`Money::checked_sub`].
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
            .expect("Money::sub: the two amounts must share a currency and fit an i64")
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decimal = self.to_decimal();
        write!(f, "{} {:?}", decimal, self.currency)
    }
}

/*
// Usage example:
fn main() {
    let price = Money::new(1990, Currency::Brl); // BRL 19.90
    let tip = Money::new(500, Currency::Brl);    // BRL 5.00

    let total = price + tip;

    println!("Total: {total}"); // 24.90 BRL

    // a 20% platform fee
    let fee = total.apply_basis_points(2_000).expect("the fee fits");

    println!("Fee: {fee}");
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_decimal_round_trips_into_the_minor_unit() {
        let amount = BigDecimal::from(1050) / BigDecimal::from(100);
        let money = Money::from_decimal(amount.clone(), Currency::Brl).expect("10.50 fits an i64");
        assert_eq!(money.amount_minor, 1050);
        assert_eq!(money.to_decimal(), amount);
    }

    #[test]
    fn a_decimal_beyond_the_i64_range_is_refused_not_panicked() {
        let absurd = BigDecimal::from(i64::MAX) * BigDecimal::from(1000);
        assert!(Money::from_decimal(absurd, Currency::Usd).is_none());
    }

    #[test]
    fn the_basis_points_of_a_share_are_computed() {
        let price = Money::new(1990, Currency::Brl);
        let fee = price.apply_basis_points(2_000).expect("the fee fits");
        assert_eq!(fee.amount_minor, 398);
    }

    #[test]
    fn the_checked_paths_refuse_mismatched_currencies() {
        let brl = Money::new(100, Currency::Brl);
        let usd = Money::new(100, Currency::Usd);
        assert!(brl.checked_add(usd).is_none());
        assert!(brl.checked_sub(usd).is_none());
    }

    #[test]
    fn the_checked_paths_refuse_an_overflowing_sum() {
        let big = Money::new(i64::MAX, Currency::Brl);
        let one = Money::new(1, Currency::Brl);
        assert!(big.checked_add(one).is_none());
    }

    #[test]
    fn the_operators_keep_the_convenient_syntax_for_one_currency() {
        let total = Money::new(1990, Currency::Brl) + Money::new(500, Currency::Brl);
        assert_eq!(total.amount_minor, 2490);
        assert_eq!(
            Money::new(2490, Currency::Brl) - Money::new(500, Currency::Brl),
            Money::new(1990, Currency::Brl)
        );
    }
}
