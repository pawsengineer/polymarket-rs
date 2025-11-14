//! # polymarket-rs
//!
//! A Rust client library for interacting with the Polymarket CLOB (Central Limit Order Book) API.
//!
//! This library provides a comprehensive, type-safe interface for:
//! - Market data queries (public)
//! - Order creation and management (authenticated)
//! - Account and balance operations (authenticated)
//! - Position tracking
//! - Real-time WebSocket streaming for market data and user events
//!
//! ## Features
//!
//! - **Builder Pattern**: Fluent API for constructing clients and orders
//! - **Type Safety**: Strong typing with newtypes for IDs (TokenId, OrderId, ConditionId)
//! - **Proper Error Handling**: No panics, comprehensive error types
//! - **EIP-712 Signing**: Full support for Ethereum wallet signatures
//! - **Decimal Precision**: Accurate decimal math for prices and amounts
//!

// Public modules
pub mod client;
pub mod config;
pub mod error;
pub mod orders;
pub mod request;
pub mod signing;
pub mod types;
pub mod websocket;

// Internal modules
mod http;
mod utils;

// Re-export commonly used types
pub use alloy_primitives::Address;
pub use alloy_signer::k256;
pub use alloy_signer_local::PrivateKeySigner;
pub use error::{Error, Result};
pub use types::{
    ApiCreds, AssetType, ConditionId, CreateOrderOptions, ExtraOrderArgs, MarketOrderArgs,
    OrderArgs, OrderId, OrderType, Side, SignatureType, TokenId,
};

// Re-export clients
pub use client::{AuthenticatedClient, ClobClient, DataClient, TradingClient};

// Re-export websocket clients
pub use websocket::{MarketWsClient, UserWsClient};

// Re-export order builder
pub use orders::OrderBuilder;

// Re-export signer trait
pub use signing::EthSigner;
