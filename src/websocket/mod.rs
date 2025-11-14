//! WebSocket clients for streaming market data and user events.
//!
//! This module provides two WebSocket clients:
//! - [`MarketWsClient`]: Streams real-time order book updates for markets
//! - [`UserWsClient`]: Streams authenticated user events (trades and order updates)
//!
//! # Connection Management
//!
//! The Polymarket WebSocket server may disconnect idle connections after 1-2 minutes.
//! For production use, it's recommended to use [`ReconnectingStream`] to automatically
//! handle disconnections and reconnect with exponential backoff.

mod market;
mod stream;
mod user;

pub use market::{MarketWsClient, SubscriptionHandle};
pub use stream::{ReconnectConfig, ReconnectingStream};
pub use user::UserWsClient;

// Re-export commonly used types for convenience
pub use crate::types::{
    BookEvent, LastTradePriceEvent, MarketSubscription, OrderEvent, PriceChange, PriceChangeEvent,
    PriceLevel, TradeEvent, UserAuthentication, UserWsEvent, WsEvent,
};
