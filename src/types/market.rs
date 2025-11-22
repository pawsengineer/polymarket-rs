use chrono::TimeDelta;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Full market information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub condition_id: String,
    pub tokens: [Token; 2],
    pub rewards: Rewards,
    pub min_incentive_size: Option<String>,
    pub max_incentive_spread: Option<String>,
    pub active: bool,
    pub closed: bool,
    pub enable_order_book: bool,
    pub archived: bool,
    pub accepting_orders: bool,
    pub accepting_order_timestamp: Option<String>,
    pub question_id: String,
    pub question: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub minimum_order_size: Decimal,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub minimum_tick_size: Decimal,
    pub description: String,
    pub category: Option<String>,
    pub end_date_iso: Option<String>,
    pub game_start_time: Option<String>,
    pub market_slug: String,
    pub icon: String,
    pub fpmm: String,
    pub neg_risk: bool,
    pub neg_risk_market_id: String,
    pub neg_risk_request_id: String,
}

impl Market {
    /// Returns true if the market ends within the specified time period from now.
    /// Returns true if there's no end date (perpetual market).
    /// Returns true if the date is invalid.
    pub fn ends_within(&self, time_delta: TimeDelta) -> bool {
        if let Some(end_date_iso) = &self.end_date_iso {
            if let Ok(end_date) = chrono::DateTime::parse_from_rfc3339(end_date_iso) {
                let now = chrono::Utc::now();
                let target_date = now + time_delta;
                return end_date <= target_date;
            }
        }
        true
    }
}

/// Simplified market information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedMarket {
    pub condition_id: String,
    pub tokens: [Token; 2],
    pub rewards: Rewards,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub accepting_orders: bool,
}

/// Token within a market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
}

/// Market rewards configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewards {
    pub rates: Option<Vec<RewardsRates>>,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub min_size: Decimal,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub max_spread: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardsRates {
    pub asset_address: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub rewards_daily_rate: Decimal,
}

/// Paginated markets response
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub limit: u64,
    pub count: u64,
    pub next_cursor: Option<String>,
    pub data: Vec<Market>,
}

/// Paginated simplified markets response
#[derive(Debug, Serialize, Deserialize)]
pub struct SimplifiedMarketsResponse {
    pub limit: u64,
    pub count: u64,
    pub next_cursor: Option<String>,
    pub data: Vec<SimplifiedMarket>,
}

/// Midpoint price response
#[derive(Debug, Deserialize, Serialize)]
pub struct MidpointResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub mid: Decimal,
}

/// Price response
#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
}

/// Price history response
#[derive(Debug, Deserialize)]
pub struct PriceHistoryResponse {
    pub history: Vec<PriceHistory>,
}

/// Price at a specific timestamp
#[derive(Debug, Deserialize)]
pub struct PriceHistory {
    #[serde(
        rename = "p",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub price: Decimal,
    #[serde(rename = "t")]
    pub timestamp: u64,
}

/// Spread response
#[derive(Debug, Deserialize)]
pub struct SpreadResponse {
    #[serde(with = "rust_decimal::serde::str")]
    pub spread: Decimal,
}

/// Tick size response
#[derive(Debug, Deserialize)]
pub struct TickSizeResponse {
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub minimum_tick_size: Decimal,
}

/// Negative risk response
#[derive(Debug, Deserialize)]
pub struct NegRiskResponse {
    pub neg_risk: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    fn create_test_market(end_date_iso: Option<String>) -> Market {
        Market {
            condition_id: "test".to_string(),
            tokens: [
                Token {
                    token_id: "token1".to_string(),
                    outcome: "Yes".to_string(),
                },
                Token {
                    token_id: "token2".to_string(),
                    outcome: "No".to_string(),
                },
            ],
            rewards: Rewards {
                rates: None,
                min_size: Decimal::ZERO,
                max_spread: Decimal::ZERO,
            },
            min_incentive_size: None,
            max_incentive_spread: None,
            active: true,
            closed: false,
            enable_order_book: true,
            archived: false,
            accepting_orders: true,
            accepting_order_timestamp: Some("2024-12-29T22:38:10Z".to_string()),
            question_id: "q1".to_string(),
            question: "Test question?".to_string(),
            minimum_order_size: Decimal::ZERO,
            minimum_tick_size: Decimal::ZERO,
            description: "Test".to_string(),
            category: None,
            end_date_iso,
            game_start_time: None,
            market_slug: "test-market".to_string(),
            icon: "".to_string(),
            fpmm: "0x0".to_string(),
            neg_risk: false,
            neg_risk_market_id: "".to_string(),
            neg_risk_request_id: "".to_string(),
        }
    }

    #[test]
    fn test_ends_within_near_future() {
        // Market ending in 1 hour should end within 2 hours
        let future_date = chrono::Utc::now() + TimeDelta::hours(1);
        let market = create_test_market(Some(future_date.to_rfc3339()));

        assert!(market.ends_within(TimeDelta::hours(2)));
    }

    #[test]
    fn test_ends_within_far_future() {
        // Market ending in 3 hours should NOT end within 1 hour
        let future_date = chrono::Utc::now() + TimeDelta::hours(3);
        let market = create_test_market(Some(future_date.to_rfc3339()));

        assert!(!market.ends_within(TimeDelta::hours(1)));
    }

    #[test]
    fn test_ends_within_no_end_date() {
        // Perpetual market (no end date) should return true
        let market = create_test_market(None);

        assert!(market.ends_within(TimeDelta::hours(24)));
    }

    #[test]
    fn test_ends_within_invalid_date() {
        // Market with invalid date format should return true (fail-open)
        let market = create_test_market(Some("invalid-date".to_string()));

        assert!(market.ends_within(TimeDelta::hours(24)));
    }

    #[test]
    fn test_ends_within_already_ended() {
        // Market that ended 1 hour ago has ended within any positive time window
        let past_date = chrono::Utc::now() - TimeDelta::hours(1);
        let market = create_test_market(Some(past_date.to_rfc3339()));

        assert!(market.ends_within(TimeDelta::hours(1)));
        assert!(market.ends_within(TimeDelta::days(7)));
    }
}
