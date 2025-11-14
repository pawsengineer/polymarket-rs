use futures_util::StreamExt;
use polymarket_rs::types::WsEvent;
use polymarket_rs::websocket::MarketWsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MarketWsClient::new();

    // Token IDs to subscribe to
    let token_ids = vec![
        // "Yes" token for "Fed decreases interest rates by 25 bps after December 2025 meeting?"
        "87769991026114894163580777793845523168226980076553814689875238288185044414090"
            .to_string(),
    ];

    println!("Connecting to CLOB WebSocket...");
    println!("Subscribing to {} token(s)", token_ids.len());

    // Subscribe to market updates
    let (mut stream, _handle) = client.subscribe_with_handle(token_ids).await?;

    println!("✅ Connected successfully!");
    println!("Waiting for events...\n");

    // Process events as they arrive
    let mut event_count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                event_count += 1;
                match event {
                    WsEvent::Book(book) => {
                        println!("[Book Event #{}]", event_count);
                        println!("  Market: {}", book.market);
                        println!("  Asset ID: {}", book.asset_id);
                        println!("  Bids: {} levels", book.bids.len());
                        if let Some(best_bid) = book.bids.first() {
                            println!("    Best bid: {} @ {}", best_bid.size, best_bid.price);
                        }
                        println!("  Asks: {} levels", book.asks.len());
                        if let Some(best_ask) = book.asks.first() {
                            println!("    Best ask: {} @ {}", best_ask.size, best_ask.price);
                        }
                        println!();
                    }
                    WsEvent::PriceChange(change) => {
                        println!("[Price Change Event #{}]", event_count);
                        println!("  Market: {}", change.market);
                        println!("  Changes: {}", change.price_changes.len());
                        for price_change in &change.price_changes {
                            println!(
                                "    {:?} @ {}: {} ({})",
                                price_change.side,
                                price_change.price,
                                price_change.size,
                                if price_change.size.is_zero() {
                                    "removed"
                                } else {
                                    "updated"
                                }
                            );
                        }
                        println!();
                    }
                    WsEvent::LastTradePrice(trade) => {
                        println!("[Trade Event #{}]", event_count);
                        println!("  Market: {}", trade.market);
                        println!("  Asset ID: {}", trade.asset_id);
                        println!("  Trade: {:?} {} @ {}", trade.side, trade.size, trade.price);
                        println!("  Fee: {} bps", trade.fee_rate_bps);
                        println!("  TX: {}", trade.transaction_hash);
                        println!();
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                break;
            }
        }
    }

    println!("WebSocket stream ended.");
    Ok(())
}
