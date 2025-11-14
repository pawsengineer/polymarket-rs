use alloy_signer_local::PrivateKeySigner;
use polymarket_rs::client::{AuthenticatedClient, TradingClient};
use polymarket_rs::orders::OrderBuilder;
use polymarket_rs::types::{CreateOrderOptions, OrderArgs, Side, SignatureType};
use polymarket_rs::Result;
use rust_decimal::Decimal;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
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
    println!("API Key: {}", api_creds.api_key);
    println!("Successfully authenticated!");

    // Step 2: Create a trading client
    println!("\n2. Setting up trading client...");
    let order_builder = OrderBuilder::new(signer.clone(), Some(SignatureType::Eoa), None);

    let trading_client = TradingClient::new(
        host,
        signer.clone(),
        chain_id,
        api_creds.clone(),
        order_builder,
    );

    // Step 3: Get existing orders
    println!("\n3. Fetching existing orders...");
    let orders = trading_client.get_orders(Default::default()).await?;
    println!("Found {} open orders", orders.data.len());

    for order in orders.data.iter().take(5) {
        println!(
            "  Order {}: {:?} {} @ {}",
            order.asset_id, order.side, order.original_size, order.price
        );
    }

    // Step 4: Create a limit order (example - NOT posted)
    println!("\n4. Creating a limit order (example)...");

    let token_id = "109648317055340591503076024421581448189531885907475125926203413622318314876012";

    let _order_args = OrderArgs::new(
        token_id,
        Decimal::from_str("0.50").unwrap(), // price
        Decimal::from_str("10.0").unwrap(), // size
        Side::Buy,
    );

    let _options = CreateOrderOptions::default()
        .tick_size(Decimal::from_str("0.01").unwrap())
        .neg_risk(false);

    // Note: This creates the order but doesn't post it
    // Uncomment the following to actually post:
    // let signed_order = trading_client.create_order(
    //     &_order_args,
    //     None,      // expiration (defaults to 0 = no expiration)
    //     None,      // extras (defaults to ExtraOrderArgs::default())
    //     _options,
    // )?;
    //
    // println!("Created signed order with salt: {}", signed_order.salt);
    //
    // // Post the order
    // let result = trading_client.post_order(signed_order, OrderType::Gtc).await?;
    // println!("Order posted: {:?}", result);

    println!("Order creation example completed (not posted)");

    // Step 5: Cancel market orders (example - NOT executed)
    println!("\n5. Cancel market orders example...");
    // Uncomment to cancel orders for a specific market:
    // let result = trading_client.cancel_market_orders(Some("0xaaa"), None).await?;
    // Or cancel by asset_id:
    // let result = trading_client.cancel_market_orders(None, Some("100")).await?;
    // Or both:
    // let result = trading_client.cancel_market_orders(Some("0xaaa"), Some("100")).await?;
    println!("Cancel market orders example completed (not executed)");

    // Step 6: Get trade history
    println!("\n6. Fetching trade history...");
    let trades = trading_client.get_trades(Default::default()).await?;
    println!("Trade history: {:?}", trades);

    println!("\n✓ Example completed successfully!");
    println!("\nNOTE: This example did not post any actual orders.");
    println!("Uncomment the posting code to execute real trades.");

    // ========================================================================
    // POLYPROXY WALLET EXAMPLE
    // ========================================================================
    // For PolyProxy wallets (email/Magic wallets), use this setup:
    //
    // use alloy_primitives::Address;
    //
    // let proxy_wallet_address = Address::from_str("0xYourProxyWalletAddress")?;
    //
    // // API authentication uses the EOA signer
    // let auth_client = AuthenticatedClient::new(
    //     host,
    //     signer.clone(),
    //     chain_id,
    //     None,
    //     Some(proxy_wallet_address),  // Pass proxy wallet address
    // );
    //
    // let api_creds = auth_client.create_or_derive_api_key().await?;
    //
    // // OrderBuilder uses PolyProxy signature type and proxy wallet as funder
    // let order_builder = OrderBuilder::new(
    //     signer.clone(),               // EOA signer (no Box::new!)
    //     Some(SignatureType::PolyProxy),
    //     Some(proxy_wallet_address),   // Proxy wallet holds funds
    // );
    //
    // let trading_client = TradingClient::new(
    //     host,
    //     signer,
    //     chain_id,
    //     api_creds,
    //     order_builder,
    // );
    //
    // // PolyProxy wallets have automatic allowance management
    // // No manual ERC-20 approvals needed!

    Ok(())
}
