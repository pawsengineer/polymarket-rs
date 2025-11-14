use crate::error::{Error, Result};
use crate::http::{create_l1_headers, create_l2_headers, HttpClient};
use crate::signing::EthSigner;
use crate::types::{ApiCreds, ApiKeysResponse, BalanceAllowanceParams};
use alloy_primitives::{Address, U256};

/// Client for authenticated operations
///
/// This client handles operations that require authentication,
/// such as API key management and account queries.
///
/// For PolyProxy wallets, the signer is used for API authentication
/// while the funder address is used as the order maker.
pub struct AuthenticatedClient {
    http_client: HttpClient,
    signer: Box<dyn EthSigner>,
    chain_id: u64,
    api_creds: Option<ApiCreds>,
    funder: Option<Address>,
}

impl AuthenticatedClient {
    /// Create a new AuthenticatedClient
    ///
    /// # Arguments
    /// * `host` - The base URL for the API
    /// * `signer` - The Ethereum signer (used for API authentication)
    /// * `chain_id` - The chain ID (137 for Polygon, 80002 for Amoy testnet)
    /// * `api_creds` - Optional API credentials for L2 operations
    /// * `funder` - Optional funder address (for PolyProxy wallets, this is the proxy wallet address)
    ///
    /// # PolyProxy Wallets
    /// For PolyProxy wallets:
    /// - `signer`: Your EOA private key (delegated signer)
    /// - `funder`: Your proxy wallet address (holds the funds)
    /// - API authentication uses the signer address
    /// - Orders are made by the funder address
    pub fn new(
        host: impl Into<String>,
        signer: impl EthSigner + 'static,
        chain_id: u64,
        api_creds: Option<ApiCreds>,
        funder: Option<Address>,
    ) -> Self {
        Self {
            http_client: HttpClient::new(host),
            signer: Box::new(signer),
            chain_id,
            api_creds,
            funder,
        }
    }

    /// Get the API credentials if available
    ///
    /// Returns a reference to the API credentials if they were provided when creating
    /// the client. This is useful for accessing credentials for WebSocket authentication.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polymarket_rs::{AuthenticatedClient, ApiCreds};
    /// # use polymarket_rs::websocket::UserWsClient;
    /// # use alloy_signer_local::PrivateKeySigner;
    /// # use futures_util::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let signer = PrivateKeySigner::random();
    /// # let creds = ApiCreds::new("key".into(), "secret".into(), "pass".into());
    /// let auth_client = AuthenticatedClient::new(
    ///     "https://clob.polymarket.com",
    ///     signer,
    ///     137,
    ///     Some(creds),
    ///     None,
    /// );
    ///
    /// // Use the credentials for WebSocket authentication
    /// if let Some(creds) = auth_client.api_creds() {
    ///     let ws_client = UserWsClient::new();
    ///     let mut stream = ws_client.subscribe_with_creds(creds).await?;
    ///     // Process events...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn api_creds(&self) -> Option<&ApiCreds> {
        self.api_creds.as_ref()
    }

    /// Create a new API key (L1 authentication required)
    ///
    /// This creates a new API key for the signer's address.
    /// Requires wallet signature.
    pub async fn create_api_key(&self, nonce: Option<U256>) -> Result<ApiCreds> {
        let headers = create_l1_headers(&self.signer, self.chain_id, nonce)?;
        self.http_client
            .post("/auth/api-key", &serde_json::json!({}), Some(headers))
            .await
    }

    /// Derive API key from existing credentials (L1 authentication required)
    pub async fn derive_api_key(&self) -> Result<ApiCreds> {
        let headers = create_l1_headers(&self.signer, self.chain_id, None)?;
        self.http_client
            .get("/auth/derive-api-key", Some(headers))
            .await
    }

    /// Create or derive API key with fallback
    ///
    /// Tries to create a new API key, falls back to derive if creation fails.
    pub async fn create_or_derive_api_key(&self) -> Result<ApiCreds> {
        match self.create_api_key(None).await {
            Ok(creds) => Ok(creds),
            Err(_) => self.derive_api_key().await,
        }
    }

    /// Get all API keys for the current user (L2 authentication required)
    pub async fn get_api_keys(&self) -> Result<ApiKeysResponse> {
        let api_creds = self
            .api_creds
            .as_ref()
            .ok_or_else(|| Error::AuthRequired("API credentials required".to_string()))?;

        let headers =
            create_l2_headers::<_, ()>(&self.signer, api_creds, "GET", "/auth/api-keys", None)?;
        self.http_client.get("/auth/api-keys", Some(headers)).await
    }

    /// Delete an API key (L2 authentication required)
    pub async fn delete_api_key(&self) -> Result<serde_json::Value> {
        let api_creds = self
            .api_creds
            .as_ref()
            .ok_or_else(|| Error::AuthRequired("API credentials required".to_string()))?;

        let headers =
            create_l2_headers::<_, ()>(&self.signer, api_creds, "DELETE", "/auth/api-key", None)?;
        self.http_client
            .delete("/auth/api-key", Some(headers))
            .await
    }

    /// Get balance and allowance information (L2 authentication required)
    ///
    /// # Arguments
    /// * `params` - Query parameters for balance/allowance
    pub async fn get_balance_allowance(
        &self,
        params: BalanceAllowanceParams,
    ) -> Result<serde_json::Value> {
        let api_creds = self
            .api_creds
            .as_ref()
            .ok_or_else(|| Error::AuthRequired("API credentials required".to_string()))?;

        // IMPORTANT: Sign the base path WITHOUT query parameters
        let base_path = "/balance-allowance";
        let headers = create_l2_headers::<_, ()>(&self.signer, api_creds, "GET", base_path, None)?;

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

    /// Update balance allowance (L2 authentication required)
    pub async fn update_balance_allowance(&self) -> Result<serde_json::Value> {
        let api_creds = self
            .api_creds
            .as_ref()
            .ok_or_else(|| Error::AuthRequired("API credentials required".to_string()))?;

        let headers = create_l2_headers::<_, ()>(
            &self.signer,
            api_creds,
            "GET",
            "/balance-allowance/update",
            None,
        )?;
        self.http_client
            .get("/balance-allowance/update", Some(headers))
            .await
    }

    /// Get notifications for the current user (L2 authentication required)
    pub async fn get_notifications(&self) -> Result<serde_json::Value> {
        let api_creds = self
            .api_creds
            .as_ref()
            .ok_or_else(|| Error::AuthRequired("API credentials required".to_string()))?;

        let headers =
            create_l2_headers::<_, ()>(&self.signer, api_creds, "GET", "/notifications", None)?;
        self.http_client.get("/notifications", Some(headers)).await
    }

    /// Drop (delete) notifications (L2 authentication required)
    pub async fn drop_notifications(&self, ids: &[String]) -> Result<serde_json::Value> {
        let api_creds = self
            .api_creds
            .as_ref()
            .ok_or_else(|| Error::AuthRequired("API credentials required".to_string()))?;

        let body = serde_json::json!({ "ids": ids });
        let headers = create_l2_headers(
            &self.signer,
            api_creds,
            "DELETE",
            "/notifications",
            Some(&body),
        )?;
        self.http_client
            .delete_with_body("/notifications", &body, Some(headers))
            .await
    }

    /// Get the signer's address
    pub fn get_address(&self) -> String {
        format!("{:?}", self.signer.address())
    }

    /// Get the funder address (for PolyProxy wallets)
    ///
    /// Returns the proxy wallet address if set, otherwise None.
    /// For EOA wallets, this should return None.
    pub fn get_funder(&self) -> Option<Address> {
        self.funder
    }
}
