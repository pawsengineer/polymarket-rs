use super::enums::{OrderType, Side};
use crate::error::Result;
use crate::{orders::calculate_market_price, OrderId};
use alloy_primitives::U256;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Arguments for creating a limit order
#[derive(Debug, Clone)]
pub struct OrderArgs {
    pub token_id: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
}

impl OrderArgs {
    pub fn new(token_id: impl Into<String>, price: Decimal, size: Decimal, side: Side) -> Self {
        Self {
            token_id: token_id.into(),
            price,
            size,
            side,
        }
    }
}

/// Arguments for creating a market order
#[derive(Debug, Clone)]
pub struct MarketOrderArgs {
    pub token_id: String,
    pub amount: Decimal,
    pub side: Side,
}

impl MarketOrderArgs {
    pub fn new(token_id: impl Into<String>, amount: Decimal, side: Side) -> Self {
        Self {
            token_id: token_id.into(),
            amount,
            side,
        }
    }
}

/// Extra optional arguments for order creation
#[derive(Debug, Clone)]
pub struct ExtraOrderArgs {
    pub fee_rate_bps: u32,
    pub nonce: U256,
    pub taker: String,
}

impl Default for ExtraOrderArgs {
    fn default() -> Self {
        Self {
            fee_rate_bps: 0,
            nonce: U256::ZERO,
            taker: ZERO_ADDRESS.into(),
        }
    }
}

impl ExtraOrderArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fee_rate_bps(mut self, fee_rate_bps: u32) -> Self {
        self.fee_rate_bps = fee_rate_bps;
        self
    }

    pub fn nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn taker(mut self, taker: impl Into<String>) -> Self {
        self.taker = taker.into();
        self
    }
}

/// Options for creating orders
#[derive(Debug, Clone, Default)]
pub struct CreateOrderOptions {
    pub tick_size: Option<Decimal>,
    pub neg_risk: Option<bool>,
}

impl CreateOrderOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick_size(mut self, tick_size: Decimal) -> Self {
        self.tick_size = Some(tick_size);
        self
    }

    pub fn neg_risk(mut self, neg_risk: bool) -> Self {
        self.neg_risk = Some(neg_risk);
        self
    }
}

/// Signed order request ready to be posted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedOrderRequest {
    pub salt: u64,
    pub maker: String,
    pub signer: String,
    pub taker: String,
    pub token_id: String,
    pub maker_amount: String,
    pub taker_amount: String,
    pub expiration: String,
    pub nonce: String,
    pub fee_rate_bps: String,
    pub side: String,
    pub signature_type: u8,
    pub signature: String,
}

/// Order to be posted to the API
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostOrder {
    order: SignedOrderRequest,
    owner: String,
    order_type: OrderType,
}

impl PostOrder {
    pub fn new(order: SignedOrderRequest, owner: String, order_type: OrderType) -> Self {
        Self {
            order,
            owner,
            order_type,
        }
    }
}

/// Response for open orders query
#[derive(Debug, Deserialize)]
pub struct OpenOrdersResponse {
    pub limit: u64,
    pub count: u64,
    pub next_cursor: Option<String>,
    pub data: Vec<OpenOrder>,
}

/// Open order from the API
#[derive(Debug, Deserialize)]
pub struct OpenOrder {
    pub id: OrderId,
    pub associate_trades: Vec<String>,
    pub status: String,
    pub market: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub original_size: Decimal,
    pub outcome: String,
    pub maker_address: String,
    pub owner: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    pub side: Side,
    #[serde(with = "rust_decimal::serde::str")]
    pub size_matched: Decimal,
    pub asset_id: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_number_from_string")]
    pub expiration: u64,
    pub order_type: OrderType,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_number_from_string")]
    pub created_at: u64,
}

/// Parameters for querying open orders
#[derive(Debug, Clone, Default)]
pub struct OpenOrderParams {
    pub id: Option<String>,
    pub asset_id: Option<String>,
    pub market: Option<String>,
}

impl OpenOrderParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn asset_id(mut self, asset_id: impl Into<String>) -> Self {
        self.asset_id = Some(asset_id.into());
        self
    }

    pub fn market(mut self, market: impl Into<String>) -> Self {
        self.market = Some(market.into());
        self
    }

    pub fn to_query_params(&self) -> Vec<(&str, &String)> {
        let mut params = Vec::with_capacity(3);

        if let Some(ref id) = self.id {
            params.push(("id", id));
        }

        if let Some(ref asset_id) = self.asset_id {
            params.push(("asset_id", asset_id));
        }

        if let Some(ref market) = self.market {
            params.push(("market", market));
        }

        params
    }
}

/// Price level in order book (price and size pair)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price at this level
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    /// Total size available at this price
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
}

/// Order book summary with bids and asks
#[derive(Debug, Deserialize)]
pub struct OrderBookSummary {
    pub market: String,
    pub asset_id: String,
    pub hash: String,
    #[serde(deserialize_with = "super::serde_helpers::deserialize_number_from_string")]
    pub timestamp: u64,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

impl OrderBookSummary {
    pub fn calculate_market_price(&self, side: Side, shares_to_match: Decimal) -> Result<Decimal> {
        calculate_market_price(
            match side {
                Side::Buy => &self.asks,
                Side::Sell => &self.bids,
            },
            shares_to_match,
            side,
        )
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    pub fn sort_bids(&self) -> Vec<PriceLevel> {
        let mut bids = self.bids.clone();
        bids.sort_by(|a, b| b.price.cmp(&a.price));
        bids
    }

    pub fn sort_asks(&self) -> Vec<PriceLevel> {
        let mut asks = self.asks.clone();
        asks.sort_by(|a, b| a.price.cmp(&b.price));
        asks
    }
}

/// Parameters for querying order book
#[derive(Debug, Serialize, Deserialize)]
pub struct BookParams {
    pub token_id: String,
    pub side: Side,
}

impl BookParams {
    pub fn new(token_id: impl Into<String>, side: Side) -> Self {
        Self {
            token_id: token_id.into(),
            side,
        }
    }
}

/// Response from posting an order
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostOrderResponse {
    pub error_msg: String,
    #[serde(rename = "orderID")]
    pub order_id: OrderId,
    pub status: String,
    pub success: bool,
}

/// Arguments for posting multiple orders
#[derive(Debug, Clone)]
pub struct PostOrderArgs {
    pub order: SignedOrderRequest,
    pub order_type: OrderType,
}

impl PostOrderArgs {
    pub fn new(order: SignedOrderRequest, order_type: OrderType) -> Self {
        Self { order, order_type }
    }
}

/// Response from canceling orders
///
/// This response is returned by:
/// - `cancel` - Cancel a single order
/// - `cancel_orders` - Cancel multiple orders
/// - `cancel_all` - Cancel all orders
/// - `cancel_market_orders` - Cancel orders by market/asset
#[derive(Debug, Deserialize)]
pub struct CancelOrdersResponse {
    pub canceled: Vec<OrderId>,
    pub not_canceled: serde_json::Value,
}
