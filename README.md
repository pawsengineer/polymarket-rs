# polymarket-rs

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

> [!NOTE]
> This library is under active development and considered alpha quality.
>
> - Breaking changes may occur in future updates without prior notice
> - API signatures, types, and module structures are subject to change
> - Not recommended for production use yet
> - Use at your own risk

A modern, type-safe Rust client library for the [Polymarket](https://polymarket.com) CLOB (Central Limit Order Book) and Data API.

This project is a complete rewrite of [polymarket-rs-client](https://github.com/TechieBoy/polymarket-rs-client) with improved ergonomics, additional API methods, and removal of generic type parameters for a cleaner API surface.

## Features

- **Full Authentication Support** - L1 (EIP-712) and L2 (HMAC) authentication
- **WebSocket Streaming** - Real-time market data and user events with automatic reconnection
- **Builder Pattern** - Fluent APIs for configuration and order creation
- **Async/Await** - Built on `tokio` for high-performance async operations
- **Decimal Precision** - Accurate financial calculations with `rust_decimal`
- **Modular Design** - Separated clients for different operations
- **Zero Panics** - Comprehensive error handling with custom `Result` types

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
polymarket-rs = { git = "https://github.com/pawsengineer/polymarket-rs.git" }
```

## Quick Start

### Client Types

| Client                | Purpose                                     | Authentication            |
| --------------------- | ------------------------------------------- | ------------------------- |
| `ClobClient`          | CLOB market data queries                    | None                      |
| `AuthenticatedClient` | API key management, account operations      | L1 (EIP-712) or L2 (HMAC) |
| `TradingClient`       | Order creation, cancellation, trade queries | L2 (HMAC)                 |
| `DataClient`          | Position and portfolio data                 | None                      |

### Public Market Data

Query market data without authentication:

```rust
use polymarket_rs::{ClobClient, TokenId};

let client = ClobClient::new("https://clob.polymarket.com");
let token_id = TokenId::new("...");

// Get midpoint price, order books, spreads, etc.
let midpoint = client.get_midpoint(&token_id).await?;
let book = client.get_order_book(&token_id).await?;
```

See [`examples/clob_data.rs`](examples/clob_data.rs) and [`examples/public_data.rs`](examples/public_data.rs) for complete examples.

### Authenticated Trading

Three-step process for authenticated trading:

```rust
use polymarket_rs::{AuthenticatedClient, TradingClient, OrderBuilder, SignatureType};

// 1. Create or derive API credentials
let auth_client = AuthenticatedClient::new(host, signer.clone(), chain_id, None, None);
let api_creds = auth_client.create_or_derive_api_key().await?;

// 2. Create trading client with order builder
let order_builder = OrderBuilder::new(signer.clone(), Some(SignatureType::Eoa), None);
let trading_client = TradingClient::new(host, signer, chain_id, api_creds, order_builder);

// 3. Create and post orders
let order_args = OrderArgs::new(token_id, price, size, Side::Buy);
trading_client.create_and_post_order(&order_args, None, None, options, OrderType::Gtc).await?;
```

**PolyProxy & PolyGnosisSafe Wallets**: For proxy wallets, pass the proxy address to `AuthenticatedClient` and use `SignatureType::PolyGnosisSafe` in `OrderBuilder`. Proxy wallets have automatic allowance management.

See [`examples/authenticated_trading.rs`](examples/authenticated_trading.rs) for complete examples including proxy wallet setup.

## WebSocket Streaming

Real-time market data and user events with automatic reconnection:

```rust
use polymarket_rs::websocket::{MarketWsClient, ReconnectingStream, ReconnectConfig};
use futures_util::StreamExt;

let client = MarketWsClient::new();
let config = ReconnectConfig::default();

let mut stream = ReconnectingStream::new(config, move || {
    client.subscribe(token_ids.clone())
});

while let Some(result) = stream.next().await {
    // Process market events
}
```

See [`examples/websocket_market.rs`](examples/websocket_market.rs) and [`examples/websocket_user.rs`](examples/websocket_user.rs) for complete streaming examples.

## Examples

Run examples from the [`examples/`](examples/) directory:

```bash
# Public market data
cargo run --example clob_data
cargo run --example public_data

# Authenticated trading
PRIVATE_KEY="0x..." cargo run --example authenticated_trading

# WebSocket streaming
cargo run --example websocket_market
PRIVATE_KEY="0x..." cargo run --example websocket_user
```

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](http://opensource.org/licenses/MIT))

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Disclaimer

This is an unofficial library and is not affiliated with Polymarket. Use at your own risk. Always test with small amounts first on testnet before using real funds.
