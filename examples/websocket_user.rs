use std::str::FromStr;
use std::time::Duration;

use futures_util::StreamExt;
use polymarket_rs::types::UserWsEvent;
use polymarket_rs::websocket::{ReconnectConfig, ReconnectingStream, UserWsClient};
use polymarket_rs::{AuthenticatedClient, PrivateKeySigner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace with your actual private key
    let private_key =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable not set");

    let signer = PrivateKeySigner::from_str(&private_key).expect("Invalid private key");

    let chain_id = 137; // Polygon Mainnet
    let host = "https://clob.polymarket.com";

    println!("Signer address: {}", signer.address());

    // Step 1: Create or derive API credentials
    println!("\n1. Creating/deriving API credentials...");

    // For EOA wallets, pass None for the funder parameter
    // For PolyProxy wallets, pass Some(proxy_wallet_address)
    let auth_client = AuthenticatedClient::new(host, signer.clone(), chain_id, None, None);

    let api_creds = auth_client.create_or_derive_api_key().await?;

    println!("Connecting to user WebSocket with authentication...");

    // Initialize the user WebSocket client
    let client = UserWsClient::new();

    // Configure automatic reconnection
    let config = ReconnectConfig {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
        max_attempts: None, // Unlimited reconnection attempts
    };

    // Create a reconnecting stream that will automatically reconnect on disconnection
    let api_creds_clone = api_creds.clone();
    let mut stream = ReconnectingStream::new(config, move || {
        let client = client.clone();
        let creds = api_creds_clone.clone();
        async move {
            println!("🔌 Connecting to user WebSocket...");
            let result = client.subscribe_with_creds(&creds).await;
            if result.is_ok() {
                println!("✅ Connected successfully!");
            }
            result
        }
    });

    println!("Connected! Waiting for events...\n");

    // Process events as they arrive
    let mut event_count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                event_count += 1;
                match event {
                    UserWsEvent::Trade(trade) => {
                        println!("[Trade Event #{}]", event_count);
                        println!("  Trade ID: {}", trade.id);
                        println!("  Market: {}", trade.market);
                        println!("  Asset ID: {}", trade.asset_id);
                        println!("  Side: {:?}", trade.side);
                        println!("  Outcome: {}", trade.outcome);
                        println!("  Price: {}", trade.price);
                        println!("  Size: {}", trade.size);
                        println!("  Status: {:?}", trade.status);
                        println!("  Maker orders: {}", trade.maker_orders.len());
                        for (i, maker) in trade.maker_orders.iter().enumerate() {
                            println!("    Maker #{}: {}", i + 1, maker.maker_address);
                            println!("      Amount: {}", maker.matched_amount);
                            println!("      Price: {}", maker.price);
                            println!("      Outcome: {}", maker.outcome);
                        }
                        println!();
                    }
                    UserWsEvent::Order(order) => {
                        println!("[Order Event #{}]", event_count);
                        println!("  Order ID: {}", order.id);
                        println!("  Market: {}", order.market);
                        println!("  Asset ID: {}", order.asset_id);
                        println!("  Side: {:?}", order.side);
                        println!("  Status: {}", order.status);
                        println!("  Event Type: {}", order.order_event_type);
                        println!("  Order Type: {}", order.order_type);
                        println!("  Outcome: {}", order.outcome);
                        println!("  Price: {}", order.price);
                        println!("  Original Size: {}", order.original_size);
                        println!("  Size Matched: {}", order.size_matched);
                        let remaining = order.original_size - order.size_matched;
                        println!("  Remaining: {}", remaining);
                        println!("  Maker Address: {}", order.maker_address);
                        println!();
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                // The ReconnectingStream will automatically attempt to reconnect
                // Continue processing to allow reconnection
                eprintln!("⏳ Will attempt to reconnect...");
            }
        }
    }

    println!("WebSocket stream ended.");
    Ok(())
}
