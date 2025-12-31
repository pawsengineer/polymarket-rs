use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::order::PriceLevel;
use super::Side;

// ============================================================================
// Market WebSocket Events
// ============================================================================

/// Websocket event from the market stream
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WsEvent {
    /// Emitted When: First subscribed to a market / when there is a trade that affects the book
    Book(BookEvent),
    /// Emitted When: A new order is placed / an order is cancelled
    PriceChange(PriceChangeEvent),
    /// Emitted When: When a maker and taker order is matched creating a trade event.
    LastTradePrice(LastTradePriceEvent),
    /// Emitted When: The minimum tick size of the market changes. This happens when the book’s price reaches the limits: price > 0.96 or price < 0.04
    TickSizeChange(TickSizeChangeEvent),
}

/// Full order book snapshot event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEvent {
    /// Event type discriminator (always "book")
    pub event_type: String,
    /// Market ID
    pub market: String,
    /// Token/Asset ID
    pub asset_id: String,
    /// Timestamp of the event
    pub timestamp: String,
    /// Hash of the order book
    pub hash: String,
    /// Buy side order book
    pub bids: Vec<PriceLevel>,
    /// Sell side order book
    pub asks: Vec<PriceLevel>,
    /// Last trade price (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_trade_price: Option<String>,
}

/// Incremental order book update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChangeEvent {
    /// Event type discriminator (always "price_change")
    pub event_type: String,
    /// Market ID
    pub market: String,
    /// Timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Hash (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// List of price changes
    pub price_changes: Vec<PriceChange>,
}

/// Individual price level change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChange {
    /// Token/Asset ID
    pub asset_id: String,
    /// Side of the book (BUY or SELL)
    pub side: Side,
    /// Price level that changed
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    /// New size at this price level (0 means remove the level)
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
}

/// Last trade price event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTradePriceEvent {
    /// Event type discriminator (always "last_trade_price")
    pub event_type: String,
    /// Market ID
    pub market: String,
    /// Token/Asset ID
    pub asset_id: String,
    /// Trade price
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    /// Trade size
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
    /// Fee rate in basis points
    #[serde(with = "rust_decimal::serde::str")]
    pub fee_rate_bps: Decimal,
    /// Side of the trade (BUY or SELL)
    pub side: Side,
    /// Timestamp of the trade
    pub timestamp: String,
    /// Transaction hash on blockchain
    pub transaction_hash: String,
}

/// Tick size change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickSizeChangeEvent {
    /// Event type discriminator (always "tick_size_change")
    pub event_type: String,
    /// Token/Asset ID
    pub asset_id: String,
    /// Market ID
    pub market: String,
    /// Previous tick size
    #[serde(with = "rust_decimal::serde::str")]
    pub old_tick_size: Decimal,
    /// New tick size
    #[serde(with = "rust_decimal::serde::str")]
    pub new_tick_size: Decimal,
    /// Timestamp of the change
    pub timestamp: String,
}

// ============================================================================
// User WebSocket Events
// ============================================================================

/// Websocket event from the authenticated user stream
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserWsEvent {
    /// Trade execution event
    Trade(TradeEvent),
    /// Order status update event
    Order(OrderEvent),
}

/// Trade execution event (when an order is matched)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    /// Event type discriminator (always "trade")
    pub event_type: String,
    /// Trade ID
    pub id: String,
    /// Market ID
    pub market: String,
    /// Token/Asset ID
    pub asset_id: String,
    /// Side of the trade (BUY or SELL)
    pub side: Side,
    /// Outcome (e.g., "Yes" or "No")
    pub outcome: String,
    /// Execution price
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    /// Execution size
    #[serde(with = "rust_decimal::serde::str")]
    pub size: Decimal,
    /// Trade status
    pub status: TradeStatus,
    /// Maker orders that were matched
    pub maker_orders: Vec<MakerOrder>,
    /// V3.9: ID of the taker order (when we are the taker)
    #[serde(default)]
    pub taker_order_id: Option<String>,
}

/// Trade execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradeStatus {
    /// Trade has been matched
    Matched,
    /// Trade has been confirmed
    Confirmed,
    /// Trade has failed
    Failed,
    /// Trade has been mined on-chain
    Mined,
}

/// Maker order that was matched in a trade
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MakerOrder {
    /// Address of the maker
    pub maker_address: String,
    /// Amount matched from this maker order
    #[serde(with = "rust_decimal::serde::str")]
    pub matched_amount: Decimal,
    /// Price of the maker order
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    /// Outcome (e.g., "Yes" or "No")
    pub outcome: String,
    /// Order ID of the maker order (for fill tracking)
    #[serde(default)]
    pub order_id: Option<String>,
    /// V2.8: Asset/Token ID of the maker order (for correct YES/NO determination)
    #[serde(default)]
    pub asset_id: Option<String>,
}

/// Order status update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEvent {
    /// Event type discriminator (always "order")
    pub event_type: String,
    /// Order ID
    pub id: String,
    /// Owner ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Market ID
    pub market: String,
    /// Token/Asset ID
    pub asset_id: String,
    /// Side of the order (BUY or SELL)
    pub side: Side,
    /// Order owner ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_owner: Option<String>,
    /// Original order size
    #[serde(with = "rust_decimal::serde::str")]
    pub original_size: Decimal,
    /// Amount that has been matched
    #[serde(with = "rust_decimal::serde::str")]
    pub size_matched: Decimal,
    /// Order price
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
    /// Associated trades (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associate_trades: Option<Vec<serde_json::Value>>,
    /// Outcome (Yes/No)
    pub outcome: String,
    /// Order event type (PLACEMENT, CANCELLATION, etc.)
    #[serde(rename = "type")]
    pub order_event_type: String,
    /// Created at timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Expiration timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<String>,
    /// Order type (GTC, FOK, etc.)
    pub order_type: String,
    /// Order status (LIVE, MATCHED, CANCELLED, etc.)
    pub status: String,
    /// Maker address
    pub maker_address: String,
    /// Event timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

// ============================================================================
// WebSocket Subscription Messages
// ============================================================================

/// Subscription message for market websocket
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MarketSubscription {
    /// List of asset/token IDs to subscribe to
    pub assets_ids: Vec<String>,
}

/// Authentication message for user websocket
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserAuthentication {
    /// Message type (always "user")
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Authentication credentials
    pub auth: AuthCredentials,
}

/// Authentication credentials for user websocket
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    /// API key
    #[serde(rename = "apiKey")]
    pub api_key: String,
    /// API secret
    pub secret: String,
    /// API passphrase
    pub passphrase: String,
}

impl UserAuthentication {
    /// Create a new authentication message
    pub fn new(api_key: String, secret: String, passphrase: String) -> Self {
        Self {
            msg_type: "user".to_string(),
            auth: AuthCredentials {
                api_key,
                secret,
                passphrase,
            },
        }
    }
}
