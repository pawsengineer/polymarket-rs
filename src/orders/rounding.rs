use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy::{AwayFromZero, MidpointTowardZero, ToZero};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

/// Rounding configuration for a specific tick size
#[derive(Debug, Clone, Copy)]
pub struct RoundConfig {
    pub price: u32,
    pub size: u32,
    pub amount: u32,
}

/// Rounding configurations for different tick sizes
pub static ROUNDING_CONFIG: LazyLock<HashMap<Decimal, RoundConfig>> = LazyLock::new(|| {
    HashMap::from([
        (
            Decimal::from_str("0.1").unwrap(),
            RoundConfig {
                price: 1,
                size: 2,
                amount: 3,
            },
        ),
        (
            Decimal::from_str("0.01").unwrap(),
            RoundConfig {
                price: 2,
                size: 2,
                amount: 4,
            },
        ),
        (
            Decimal::from_str("0.001").unwrap(),
            RoundConfig {
                price: 3,
                size: 2,
                amount: 5,
            },
        ),
        (
            Decimal::from_str("0.0001").unwrap(),
            RoundConfig {
                price: 4,
                size: 2,
                amount: 6,
            },
        ),
    ])
});

/// Convert decimal amount to token units (multiply by 1e6 and round)
pub fn decimal_to_token_u64(amt: Decimal) -> u64 {
    let mut amt = Decimal::from_scientific("1e6").expect("1e6 is not scientific") * amt;
    if amt.scale() > 0 {
        amt = amt.round_dp_with_strategy(0, MidpointTowardZero);
    }
    amt.try_into().expect("Couldn't round decimal to integer")
}

/// Fix amount rounding to ensure proper precision
pub fn fix_amount_rounding(mut amt: Decimal, round_config: &RoundConfig) -> Decimal {
    if amt.scale() > round_config.amount {
        amt = amt.round_dp_with_strategy(round_config.amount + 4, AwayFromZero);
        if amt.scale() > round_config.amount {
            amt = amt.round_dp_with_strategy(round_config.amount, ToZero);
        }
    }
    amt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounding_configs_exist() {
        assert!(ROUNDING_CONFIG.contains_key(&Decimal::from_str("0.1").unwrap()));
        assert!(ROUNDING_CONFIG.contains_key(&Decimal::from_str("0.01").unwrap()));
        assert!(ROUNDING_CONFIG.contains_key(&Decimal::from_str("0.001").unwrap()));
        assert!(ROUNDING_CONFIG.contains_key(&Decimal::from_str("0.0001").unwrap()));
    }

    #[test]
    fn test_decimal_to_token() {
        let result = decimal_to_token_u64(Decimal::from_str("1.5").unwrap());
        assert_eq!(result, 1_500_000);
    }
}
