use crate::error::Result;
use crate::http::{create_l2_headers, HttpClient};
use crate::orders::OrderBuilder;
use crate::signing::EthSigner;
use crate::types::{
    ApiCreds, CancelOrdersResponse, CreateOrderOptions, ExtraOrderArgs, MarketOrderArgs,
    OpenOrder, OpenOrderParams, OpenOrdersResponse, OrderArgs, OrderBookSummary, OrderId,
    OrderType, PostOrder, PostOrderResponse, Side, SignedOrderRequest, TradeParams,
};

/// Client for trading operations
///
/// This client handles order creation, cancellation, and trade queries.
/// All operations require L2 authentication (API credentials).
pub struct TradingClient {
    http_client: HttpClient,
    signer: Box<dyn EthSigner>,
    chain_id: u64,
    api_creds: ApiCreds,
    order_builder: OrderBuilder,
}

impl TradingClient {
    /// Create a new TradingClient
    ///
    /// # Arguments
    /// * `host` - The base URL for the API
    /// * `signer` - The Ethereum signer
    /// * `chain_id` - The chain ID (137 for Polygon, 80002 for Amoy testnet)
    /// * `api_creds` - API credentials for authentication
    /// * `order_builder` - OrderBuilder instance for creating orders
    pub fn new(
        host: impl Into<String>,
        signer: impl EthSigner + 'static,
        chain_id: u64,
        api_creds: ApiCreds,
        order_builder: OrderBuilder,
    ) -> Self {
        Self {
            http_client: HttpClient::new(host),
            signer: Box::new(signer),
            chain_id,
            api_creds,
            order_builder,
        }
    }

    /// Create a limit order (local operation, not posted)
    ///
    /// # Arguments
    /// * `order_args` - Order arguments (token_id, price, size, side)
    /// * `expiration` - Optional expiration timestamp (defaults to 0 = no expiration)
    /// * `extras` - Optional extra order parameters (defaults to ExtraOrderArgs::default())
    /// * `options` - Order options (tick_size, neg_risk must be provided)
    pub fn create_order(
        &self,
        order_args: &OrderArgs,
        expiration: Option<u64>,
        extras: Option<&ExtraOrderArgs>,
        options: CreateOrderOptions,
    ) -> Result<SignedOrderRequest> {
        let expiration = expiration.unwrap_or(0);
        let default_extras = ExtraOrderArgs::default();
        let extras = extras.unwrap_or(&default_extras);

        self.order_builder
            .create_order(self.chain_id, order_args, expiration, extras, options)
    }

    /// Create a market order (local operation, not posted)
    ///
    /// # Arguments
    /// * `order_args` - Market order arguments (token_id, amount, side)
    /// * `order_book` - The order book to calculate price from
    /// * `extras` - Optional extra order parameters (defaults to ExtraOrderArgs::default())
    /// * `options` - Order options (tick_size, neg_risk must be provided)
    pub fn create_market_order(
        &self,
        order_args: &MarketOrderArgs,
        order_book: &OrderBookSummary,
        extras: Option<&ExtraOrderArgs>,
        options: CreateOrderOptions,
    ) -> Result<SignedOrderRequest> {
        let default_extras = ExtraOrderArgs::default();
        let extras = extras.unwrap_or(&default_extras);

        // Use asks for BUY (taking from sellers), bids for SELL (taking from buyers)
        let book_side = match order_args.side {
            Side::Buy => &order_book.asks,
            Side::Sell => &order_book.bids,
        };

        // Calculate market price from order book
        let price = self
            .order_builder
            .calculate_market_price(book_side, order_args.amount)?;

        self.order_builder
            .create_market_order(self.chain_id, order_args, price, extras, options)
    }

    /// Post an order to the exchange
    ///
    /// # Arguments
    /// * `order` - The signed order to post
    /// * `order_type` - The order type (GTC, FOK, GTD)
    pub async fn post_order(
        &self,
        order: SignedOrderRequest,
        order_type: OrderType,
    ) -> Result<PostOrderResponse> {
        let owner = self.api_creds.api_key.clone();
        let post_order = PostOrder::new(order, owner, order_type);

        let headers = create_l2_headers(
            &self.signer,
            &self.api_creds,
            "POST",
            "/order",
            Some(&post_order),
        )?;
        self.http_client
            .post("/order", &post_order, Some(headers))
            .await
    }

    /// Create and post an order in one step
    ///
    /// This is a convenience method that combines create_order and post_order.
    ///
    /// # Arguments
    /// * `order_args` - Order arguments (token_id, price, size, side)
    /// * `expiration` - Optional expiration timestamp (defaults to 0 = no expiration)
    /// * `extras` - Optional extra order parameters (defaults to ExtraOrderArgs::default())
    /// * `options` - Order options (tick_size, neg_risk must be provided)
    /// * `order_type` - The order type (GTC, FOK, GTD)
    pub async fn create_and_post_order(
        &self,
        order_args: &OrderArgs,
        expiration: Option<u64>,
        extras: Option<&ExtraOrderArgs>,
        options: CreateOrderOptions,
        order_type: OrderType,
    ) -> Result<PostOrderResponse> {
        let order = self.create_order(order_args, expiration, extras, options)?;
        self.post_order(order, order_type).await
    }

    /// Get open orders (L2 authentication required)
    ///
    /// # Arguments
    /// * `params` - Query parameters to filter orders
    pub async fn get_orders(&self, params: OpenOrderParams) -> Result<OpenOrdersResponse> {
        // IMPORTANT: Sign the base path WITHOUT query parameters
        // Query parameters are added to the URL after signing
        let base_path = "/data/orders";
        let headers =
            create_l2_headers::<_, ()>(&self.signer, &self.api_creds, "GET", base_path, None)?;

        // Build the full request path WITH query parameters
        let query_params = params.to_query_params();
        let request_path = if query_params.is_empty() {
            base_path.to_string()
        } else {
            format!(
                "{}?{}",
                base_path,
                query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&")
            )
        };

        self.http_client.get(&request_path, Some(headers)).await
    }

    /// Get a specific order by ID
    pub async fn get_order(&self, order_id: &OrderId) -> Result<OpenOrder> {
        let path = format!("/data/order/{}", order_id.as_str());
        let headers =
            create_l2_headers::<_, ()>(&self.signer, &self.api_creds, "GET", &path, None)?;
        self.http_client.get(&path, Some(headers)).await
    }

    /// Cancel a specific order
    ///
    /// # Arguments
    /// * `order_id` - The ID of the order to cancel
    pub async fn cancel(&self, order_id: &OrderId) -> Result<CancelOrdersResponse> {
        let body = serde_json::json!({ "orderID": order_id.as_str() });
        let headers = create_l2_headers(
            &self.signer,
            &self.api_creds,
            "DELETE",
            "/order",
            Some(&body),
        )?;
        self.http_client
            .delete_with_body("/order", &body, Some(headers))
            .await
    }

    /// Cancel multiple orders
    ///
    /// # Arguments
    /// * `order_ids` - List of order IDs to cancel
    pub async fn cancel_orders(&self, order_ids: &[OrderId]) -> Result<CancelOrdersResponse> {
        let ids: Vec<&str> = order_ids.iter().map(|id| id.as_str()).collect();
        let body = serde_json::json!(ids);
        let headers = create_l2_headers(
            &self.signer,
            &self.api_creds,
            "DELETE",
            "/orders",
            Some(&body),
        )?;
        self.http_client
            .delete_with_body("/orders", &body, Some(headers))
            .await
    }

    /// Cancel all orders
    pub async fn cancel_all(&self) -> Result<CancelOrdersResponse> {
        let body = serde_json::json!({});
        let headers = create_l2_headers(
            &self.signer,
            &self.api_creds,
            "DELETE",
            "/cancel-all",
            Some(&body),
        )?;
        self.http_client
            .delete_with_body("/cancel-all", &body, Some(headers))
            .await
    }

    /// Cancel all orders for a specific market and/or asset
    ///
    /// # Arguments
    /// * `market` - Optional market to cancel orders for (None = empty string)
    /// * `asset_id` - Optional asset ID to cancel orders for (None = empty string)
    pub async fn cancel_market_orders(
        &self,
        market: Option<&str>,
        asset_id: Option<&str>,
    ) -> Result<CancelOrdersResponse> {
        // Python SDK always sends both fields, defaulting to empty strings
        let body = serde_json::json!({
            "market": market.unwrap_or(""),
            "asset_id": asset_id.unwrap_or("")
        });

        let headers = create_l2_headers(
            &self.signer,
            &self.api_creds,
            "DELETE",
            "/cancel-market-orders",
            Some(&body),
        )?;
        self.http_client
            .delete_with_body("/cancel-market-orders", &body, Some(headers))
            .await
    }

    /// Get trade history (L2 authentication required)
    ///
    /// # Arguments
    /// * `params` - Query parameters to filter trades
    pub async fn get_trades(&self, params: TradeParams) -> Result<serde_json::Value> {
        // IMPORTANT: Sign the base path WITHOUT query parameters
        let base_path = "/data/trades";
        let headers =
            create_l2_headers::<_, ()>(&self.signer, &self.api_creds, "GET", base_path, None)?;

        // Build the full request path WITH query parameters
        let query_params = params.to_query_params();
        let request_path = if query_params.is_empty() {
            base_path.to_string()
        } else {
            format!(
                "{}?{}",
                base_path,
                query_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&")
            )
        };

        self.http_client.get(&request_path, Some(headers)).await
    }

    /// Check if an order is scoring
    pub async fn is_order_scoring(&self, order_id: &OrderId) -> Result<serde_json::Value> {
        // IMPORTANT: Sign the base path WITHOUT query parameters
        let base_path = "/order-scoring";
        let headers =
            create_l2_headers::<_, ()>(&self.signer, &self.api_creds, "GET", base_path, None)?;

        // Build the full request path WITH query parameters
        let request_path = format!("{}?id={}", base_path, order_id.as_str());

        self.http_client.get(&request_path, Some(headers)).await
    }

    /// Check if multiple orders are scoring
    pub async fn are_orders_scoring(&self, order_ids: &[OrderId]) -> Result<serde_json::Value> {
        let ids: Vec<&str> = order_ids.iter().map(|id| id.as_str()).collect();
        let body = serde_json::json!(ids);
        let headers = create_l2_headers(
            &self.signer,
            &self.api_creds,
            "POST",
            "/orders-scoring",
            Some(&body),
        )?;
        self.http_client
            .post("/orders-scoring", &body, Some(headers))
            .await
    }
}
