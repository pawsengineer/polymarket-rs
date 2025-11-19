use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{types::ActivityType, Side};

/// Custom deserializer for optional Side that treats empty strings as None
fn deserialize_optional_side<'de, D>(deserializer: D) -> Result<Option<Side>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        serde_json::from_str(&format!("\"{}\"", s))
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

/// User position information
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Position {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub asset: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub size: Decimal,
    #[serde(
        rename = "avgPrice",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub avg_price: Decimal,
    #[serde(
        rename = "initialValue",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub initial_value: Decimal,
    #[serde(
        rename = "currentValue",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub current_value: Decimal,
    #[serde(
        rename = "cashPnl",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub cash_pnl: Decimal,
    #[serde(
        rename = "percentPnl",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub percent_pnl: Decimal,
    #[serde(
        rename = "totalBought",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub total_bought: Decimal,
    #[serde(
        rename = "realizedPnl",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub realized_pnl: Decimal,
    #[serde(
        rename = "percentRealizedPnl",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub percent_realized_pnl: Decimal,
    #[serde(
        rename = "curPrice",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub cur_price: Decimal,
    pub redeemable: bool,
    pub mergeable: bool,
    pub title: String,
    #[serde(rename = "eventId")]
    pub event_id: String,
    pub outcome: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u32,
    #[serde(rename = "oppositeOutcome")]
    pub opposite_outcome: String,
    #[serde(rename = "oppositeAsset")]
    pub opposite_asset: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
    #[serde(rename = "negativeRisk")]
    pub negative_risk: bool,
}

/// User position value summary
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PositionValue {
    pub user: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub value: Decimal,
}

/// Trade information from the data API
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Trade {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub side: Side,
    pub asset: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub size: Decimal,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub price: Decimal,
    pub timestamp: u64,
    pub title: String,
    pub slug: String,
    pub icon: String,
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    pub outcome: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u32,
    pub name: String,
    pub pseudonym: String,
    pub bio: String,
    #[serde(rename = "profileImage")]
    pub profile_image: String,
    #[serde(rename = "profileImageOptimized")]
    pub profile_image_optimized: String,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
}

/// Activity information from the data API
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Activity {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub timestamp: u64,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub size: Decimal,
    #[serde(
        rename = "usdcSize",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub usdc_size: Decimal,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_decimal")]
    pub price: Decimal,
    pub asset: String,
    #[serde(default, deserialize_with = "deserialize_optional_side")]
    pub side: Option<Side>,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u32,
    pub title: String,
    pub slug: String,
    pub icon: String,
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    pub outcome: String,
    pub name: String,
}

/// Closed position information from the data API
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClosedPosition {
    #[serde(rename = "proxyWallet")]
    pub proxy_wallet: String,
    pub asset: String,
    #[serde(rename = "conditionId")]
    pub condition_id: String,
    #[serde(
        rename = "avgPrice",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub avg_price: Decimal,
    #[serde(
        rename = "totalBought",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub total_bought: Decimal,
    #[serde(
        rename = "realizedPnl",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub realized_pnl: Decimal,
    #[serde(
        rename = "curPrice",
        deserialize_with = "super::serde_helpers::deserialize_decimal"
    )]
    pub cur_price: Decimal,
    pub timestamp: u64,
    pub title: String,
    pub slug: String,
    pub icon: String,
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    pub outcome: String,
    #[serde(rename = "outcomeIndex")]
    pub outcome_index: u32,
    #[serde(rename = "oppositeOutcome")]
    pub opposite_outcome: String,
    #[serde(rename = "oppositeAsset")]
    pub opposite_asset: String,
    #[serde(rename = "endDate")]
    pub end_date: String,
}

/// Parameters for querying trades
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TradeParams {
    pub id: Option<String>,
    pub maker_address: Option<String>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub before: Option<u64>,
    pub after: Option<u64>,
}

impl TradeParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn maker_address(mut self, maker_address: impl Into<String>) -> Self {
        self.maker_address = Some(maker_address.into());
        self
    }

    pub fn market(mut self, market: impl Into<String>) -> Self {
        self.market = Some(market.into());
        self
    }

    pub fn asset_id(mut self, asset_id: impl Into<String>) -> Self {
        self.asset_id = Some(asset_id.into());
        self
    }

    pub fn before(mut self, before: u64) -> Self {
        self.before = Some(before);
        self
    }

    pub fn after(mut self, after: u64) -> Self {
        self.after = Some(after);
        self
    }

    pub fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::with_capacity(6);

        if let Some(ref id) = self.id {
            params.push(("id", id.clone()));
        }

        if let Some(ref asset_id) = self.asset_id {
            params.push(("asset_id", asset_id.clone()));
        }

        if let Some(ref market) = self.market {
            params.push(("market", market.clone()));
        }

        if let Some(before) = self.before {
            params.push(("before", before.to_string()));
        }

        if let Some(after) = self.after {
            params.push(("after", after.to_string()));
        }

        if let Some(ref maker_address) = self.maker_address {
            params.push(("maker_address", maker_address.clone()));
        }

        params
    }
}
