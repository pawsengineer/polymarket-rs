use crate::error::Result;
use crate::http::HttpClient;
use crate::request::PaginationParams;
use crate::types::{
    BookParams, ConditionId, Market, MarketsResponse, MidpointResponse, NegRiskResponse,
    OrderBookSummary, PriceHistoryResponse, PriceResponse, SimplifiedMarketsResponse,
    SpreadResponse, TickSizeResponse, TokenId,
};
use crate::Side;

/// Client for CLOB (Central Limit Order Book) market data APIs
///
/// This client provides access to all public CLOB market data endpoints
/// without requiring authentication.
pub struct ClobClient {
    http_client: HttpClient,
}

impl ClobClient {
    /// Create a new ClobClient
    ///
    /// # Arguments
    /// * `host` - The base URL for the API (e.g., "https://clob.polymarket.com")
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            http_client: HttpClient::new(host),
        }
    }

    /// Check if the server is responsive
    pub async fn get_ok(&self) -> Result<serde_json::Value> {
        self.http_client.get("/", None).await
    }

    /// Get current server time
    pub async fn get_server_time(&self) -> Result<serde_json::Value> {
        self.http_client.get("/time", None).await
    }

    /// Get the midpoint price for a token
    ///
    /// # Arguments
    /// * `token_id` - The token ID to query
    pub async fn get_midpoint(&self, token_id: &TokenId) -> Result<MidpointResponse> {
        let path = format!("/midpoint?token_id={}", token_id.as_str());
        self.http_client.get(&path, None).await
    }

    /// Get midpoint prices for multiple tokens
    ///
    /// # Arguments
    /// * `token_ids` - List of token IDs to query
    pub async fn get_midpoints(&self, token_ids: &[TokenId]) -> Result<Vec<MidpointResponse>> {
        let ids: Vec<&str> = token_ids.iter().map(|id| id.as_str()).collect();
        self.http_client
            .post("/midpoints", &serde_json::json!({ "token_ids": ids }), None)
            .await
    }

    /// Get the current price for a token
    ///
    /// # Arguments
    /// * `token_id` - The token ID to query
    /// * `side` - Optional side (BUY or SELL)
    pub async fn get_price(&self, token_id: &TokenId, side: Side) -> Result<PriceResponse> {
        let mut path = format!("/price?token_id={}", token_id.as_str());
        path.push_str(&format!("&side={}", side.as_str()));
        self.http_client.get(&path, None).await
    }

    /// Get prices for multiple tokens
    pub async fn get_prices(&self, token_ids: &[TokenId]) -> Result<Vec<PriceResponse>> {
        let ids: Vec<&str> = token_ids.iter().map(|id| id.as_str()).collect();
        self.http_client
            .post("/prices", &serde_json::json!({ "token_ids": ids }), None)
            .await
    }

    /// Get price history for a token
    ///
    /// # Arguments
    /// * `token_id` - The token ID to query
    /// * `start_ts` - Optional start timestamp (seconds)
    /// * `end_ts` - Optional end timestamp (seconds)
    /// * `interval` - Optional interval (e.g., "1m", "5m", "1h")
    /// * `fidelity` - Optional fidelity (the resolution of the data, in minutes)
    pub async fn get_prices_history(
        &self,
        token_id: &TokenId,
        interval: &str,
        start_ts: Option<u64>,
        end_ts: Option<u64>,
        fidelity: Option<u64>,
    ) -> Result<PriceHistoryResponse> {
        let mut path = format!("/prices-history?market={}", token_id.as_str());
        path.push_str(&format!("&interval={}", interval));
        path.push_str(&format!("&fidelity={}", fidelity.unwrap_or(10)));
        if let Some(start) = start_ts {
            path.push_str(&format!("&startTs={}", start));
        }
        if let Some(end) = end_ts {
            path.push_str(&format!("&endTs={}", end));
        }
        self.http_client.get(&path, None).await
    }

    /// Get the bid/ask spread for a token
    pub async fn get_spread(&self, token_id: &TokenId) -> Result<SpreadResponse> {
        let path = format!("/spread?token_id={}", token_id.as_str());
        self.http_client.get(&path, None).await
    }

    /// Get spreads for multiple tokens
    pub async fn get_spreads(&self, token_ids: &[TokenId]) -> Result<Vec<SpreadResponse>> {
        let ids: Vec<&str> = token_ids.iter().map(|id| id.as_str()).collect();
        self.http_client
            .post("/spreads", &serde_json::json!({ "token_ids": ids }), None)
            .await
    }

    /// Get the minimum tick size for a token
    pub async fn get_tick_size(&self, token_id: &TokenId) -> Result<TickSizeResponse> {
        let path = format!("/tick-size?token_id={}", token_id.as_str());
        self.http_client.get(&path, None).await
    }

    /// Get whether a market uses negative risk
    pub async fn get_neg_risk(&self, condition_id: &ConditionId) -> Result<NegRiskResponse> {
        let path = format!("/neg-risk?condition_id={}", condition_id.as_str());
        self.http_client.get(&path, None).await
    }

    /// Get the order book for a token
    ///
    /// # Arguments
    /// * `token_id` - The token ID to query
    pub async fn get_order_book(&self, token_id: &TokenId) -> Result<OrderBookSummary> {
        let path = format!("/book?token_id={}", token_id.as_str());
        self.http_client.get(&path, None).await
    }

    /// Get order books for multiple tokens
    pub async fn get_order_books(&self, params: &[BookParams]) -> Result<Vec<OrderBookSummary>> {
        self.http_client.post("/books", &params, None).await
    }

    /// Get the last trade price for a token
    pub async fn get_last_trade_price(&self, token_id: &TokenId) -> Result<PriceResponse> {
        let path = format!("/last-trade-price?token_id={}", token_id.as_str());
        self.http_client.get(&path, None).await
    }

    /// Get last trade prices for multiple tokens
    pub async fn get_last_trade_prices(&self, token_ids: &[TokenId]) -> Result<serde_json::Value> {
        let ids: Vec<&str> = token_ids.iter().map(|id| id.as_str()).collect();
        self.http_client
            .post(
                "/last-trades-prices",
                &serde_json::json!({ "token_ids": ids }),
                None,
            )
            .await
    }

    /// Get sampling markets with pagination
    ///
    /// # Arguments
    /// * `pagination` - Pagination parameters
    pub async fn get_sampling_markets(
        &self,
        pagination: Option<PaginationParams>,
    ) -> Result<MarketsResponse> {
        let mut path = "/sampling-markets".to_string();
        if let Some(p) = pagination {
            let params = p.to_query_params();
            if !params.is_empty() {
                path.push_str("?next_cursor=");
                path.push_str(&params[0].1);
            }
        }
        self.http_client.get(&path, None).await
    }

    /// Get sampling simplified markets with pagination
    ///
    /// # Arguments
    /// * `pagination` - Pagination parameters
    pub async fn get_sampling_simplified_markets(
        &self,
        pagination: Option<PaginationParams>,
    ) -> Result<SimplifiedMarketsResponse> {
        let mut path = "/sampling-simplified-markets".to_string();
        if let Some(p) = pagination {
            let params = p.to_query_params();
            if !params.is_empty() {
                path.push_str("?next_cursor=");
                path.push_str(&params[0].1);
            }
        }
        self.http_client.get(&path, None).await
    }

    /// Get markets with pagination
    ///
    /// # Arguments
    /// * `pagination` - Pagination parameters
    pub async fn get_markets(
        &self,
        pagination: Option<PaginationParams>,
    ) -> Result<MarketsResponse> {
        let mut path = "/markets".to_string();
        if let Some(p) = pagination {
            let params = p.to_query_params();
            if !params.is_empty() {
                path.push_str("?next_cursor=");
                path.push_str(&params[0].1);
            }
        }
        self.http_client.get(&path, None).await
    }

    /// Get simplified markets with pagination
    pub async fn get_simplified_markets(
        &self,
        pagination: Option<PaginationParams>,
    ) -> Result<SimplifiedMarketsResponse> {
        let mut path = "/simplified-markets".to_string();
        if let Some(p) = pagination {
            let params = p.to_query_params();
            if !params.is_empty() {
                path.push_str("?next_cursor=");
                path.push_str(&params[0].1);
            }
        }
        self.http_client.get(&path, None).await
    }

    /// Get a specific market by condition ID
    pub async fn get_market(&self, condition_id: &ConditionId) -> Result<Market> {
        let path = format!("/markets/{}", condition_id.as_str());
        self.http_client.get(&path, None).await
    }

    /// Get a specific market by slug
    pub async fn get_market_by_slug(&self, market_slug: &str) -> Result<Market> {
        let path = format!("/markets/slug/{}", market_slug);
        self.http_client.get(&path, None).await
    }

    /// Get live activity events for a market (trades and events)
    ///
    /// # Arguments
    /// * `condition_id` - The condition ID of the market
    pub async fn get_market_trades_events(
        &self,
        condition_id: &ConditionId,
    ) -> Result<serde_json::Value> {
        let path = format!("/live-activity/events/{}", condition_id.as_str());
        self.http_client.get(&path, None).await
    }
}
