use futures_util::{SinkExt, Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::{Error, Result};
use crate::types::{MarketSubscription, WsEvent};

/// Handle for querying WebSocket subscription state
///
/// This handle provides read-only access to the current token IDs
/// being subscribed to.
///
/// **Note**: Polymarket does not support updating subscriptions on an
/// existing WebSocket connection. To change subscriptions, you must
/// close the connection and create a new one with the updated token list.
#[derive(Clone)]
pub struct SubscriptionHandle {
    /// Shared state containing current token IDs
    current_tokens: Arc<RwLock<Vec<String>>>,
}

impl SubscriptionHandle {
    /// Get the current token IDs being subscribed to
    pub async fn current_tokens(&self) -> Vec<String> {
        self.current_tokens.read().await.clone()
    }
}

/// WebSocket client for streaming market data (order book updates)
///
/// This client connects to the Polymarket CLOB WebSocket endpoint and streams
/// real-time order book updates for specified token IDs.
///
/// # Connection Management
///
/// The Polymarket WebSocket server will disconnect idle connections after 1-2 minutes.
/// The Python client uses `ping_interval=5` to send keep-alive pings every 5 seconds.
///
/// For Rust, the recommended approach is to use [`ReconnectingStream`](crate::websocket::ReconnectingStream)
/// which automatically handles connection resets and reconnects with exponential backoff.
/// This is more robust than manual ping/pong management.
#[derive(Debug, Clone)]
pub struct MarketWsClient {
    ws_url: String,
}

impl MarketWsClient {
    /// Default WebSocket URL for market data
    const DEFAULT_WS_URL: &'static str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

    /// Create a new market WebSocket client with the default endpoint
    pub fn new() -> Self {
        Self {
            ws_url: Self::DEFAULT_WS_URL.to_string(),
        }
    }

    /// Create a new market WebSocket client with a custom endpoint
    pub fn with_url(ws_url: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
        }
    }

    /// Subscribe to market updates with a handle to query subscription state
    ///
    /// Returns a stream of [`WsEvent`] items and a [`SubscriptionHandle`] that can be used
    /// to query which token IDs are currently subscribed.
    ///
    /// **Note**: Polymarket does not support updating subscriptions on an existing connection.
    /// To change subscriptions, you must close the connection and create a new one.
    ///
    /// # Arguments
    ///
    /// * `token_ids` - List of token/asset IDs to subscribe to
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Stream of [`WsEvent`] items
    /// - [`SubscriptionHandle`] for querying current subscriptions
    ///
    /// # Events
    ///
    /// The stream will yield three types of events:
    /// - [`WsEvent::Book`]: Full order book snapshot (sent initially)
    /// - [`WsEvent::PriceChange`]: Incremental updates to the order book
    /// - [`WsEvent::LastTradePrice`]: Trade execution events
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The WebSocket connection fails
    /// - The subscription message cannot be sent
    pub async fn subscribe_with_handle(
        &self,
        token_ids: Vec<String>,
    ) -> Result<(
        Pin<Box<dyn Stream<Item = Result<WsEvent>> + Send>>,
        SubscriptionHandle,
    )> {
        // Connect to the WebSocket endpoint
        let (ws_stream, _) = connect_async(&self.ws_url).await?;

        let (write, read) = ws_stream.split();
        let mut write = write;

        // Create subscription message
        let subscription = MarketSubscription {
            assets_ids: token_ids.clone(),
        };

        let subscription_msg = serde_json::to_string(&subscription)?;

        // Send initial subscription message
        write
            .send(Message::Text(subscription_msg))
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;

        // Drop the write half since we don't need to send any more messages
        drop(write);

        // Create shared state for current tokens
        let current_tokens = Arc::new(RwLock::new(token_ids));

        // Create subscription handle
        let handle = SubscriptionHandle { current_tokens };

        // Return stream that parses events
        let stream = read.filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(text)) => {
                    // Skip empty or whitespace-only messages
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        return None;
                    }

                    // Skip PING/PONG messages sent as text (some servers do this)
                    if trimmed.eq_ignore_ascii_case("ping") || trimmed.eq_ignore_ascii_case("pong") {
                        return None;
                    }

                    // The server can send either a single object or an array
                    // Try to parse as array first
                    if let Ok(events) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                        // Got an array, take the first event
                        if let Some(first) = events.first() {
                            match serde_json::from_value::<WsEvent>(first.clone()) {
                                Ok(event) => return Some(Ok(event)),
                                Err(e) => return Some(Err(Error::Json(e))),
                            }
                        } else {
                            // Empty array, ignore
                            return None;
                        }
                    }

                    // Try parsing as single object
                    match serde_json::from_str::<WsEvent>(&text) {
                        Ok(event) => Some(Ok(event)),
                        Err(e) => {
                            // Log unexpected message format for debugging
                            eprintln!("Unexpected WebSocket message (first 200 chars): {}",
                                     &text.chars().take(200).collect::<String>());
                            Some(Err(Error::Json(e)))
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    // Connection closed gracefully
                    Some(Err(Error::ConnectionClosed))
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // Ignore ping/pong frames (handled automatically)
                    None
                }
                Ok(Message::Binary(_)) => {
                    // Unexpected binary message
                    Some(Err(Error::WebSocket(
                        "Unexpected binary message".to_string(),
                    )))
                }
                Ok(Message::Frame(_)) => {
                    // Raw frame (shouldn't happen)
                    None
                }
                Err(e) => {
                    // WebSocket error
                    Some(Err(Error::WebSocket(e.to_string())))
                }
            }
        });

        Ok((Box::pin(stream), handle))
    }

    /// Subscribe to market updates for the specified token IDs
    ///
    /// Returns a stream of [`WsEvent`] items. The stream will yield events as they
    /// are received from the WebSocket connection.
    ///
    /// **Note:** This method does not support dynamic subscription updates.
    /// Use [`subscribe_with_handle`](Self::subscribe_with_handle) if you need to
    /// update subscriptions without reconnecting.
    ///
    /// # Arguments
    ///
    /// * `token_ids` - List of token/asset IDs to subscribe to
    ///
    /// # Events
    ///
    /// The stream will yield three types of events:
    /// - [`WsEvent::Book`]: Full order book snapshot (sent initially)
    /// - [`WsEvent::PriceChange`]: Incremental updates to the order book
    /// - [`WsEvent::LastTradePrice`]: Trade execution events
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The WebSocket connection fails
    /// - The subscription message cannot be sent
    pub async fn subscribe(
        &self,
        token_ids: Vec<String>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsEvent>> + Send>>> {
        // Connect to the WebSocket endpoint
        let (ws_stream, _) = connect_async(&self.ws_url).await?;

        let (write, read) = ws_stream.split();
        let mut write = write;

        // Create subscription message
        let subscription = MarketSubscription {
            assets_ids: token_ids,
        };

        let subscription_msg = serde_json::to_string(&subscription)?;

        // Send subscription message
        write
            .send(Message::Text(subscription_msg))
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;

        // Drop the write half since we don't need to send any more messages
        drop(write);

        // Return stream that parses events
        let stream = read.filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(text)) => {
                    // Skip empty or whitespace-only messages
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        return None;
                    }

                    // Skip PING/PONG messages sent as text (some servers do this)
                    if trimmed.eq_ignore_ascii_case("ping") || trimmed.eq_ignore_ascii_case("pong") {
                        return None;
                    }

                    // The server can send either a single object or an array
                    // Try to parse as array first
                    if let Ok(events) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                        // Got an array, take the first event
                        if let Some(first) = events.first() {
                            match serde_json::from_value::<WsEvent>(first.clone()) {
                                Ok(event) => return Some(Ok(event)),
                                Err(e) => return Some(Err(Error::Json(e))),
                            }
                        } else {
                            // Empty array, ignore
                            return None;
                        }
                    }

                    // Try parsing as single object
                    match serde_json::from_str::<WsEvent>(&text) {
                        Ok(event) => Some(Ok(event)),
                        Err(e) => {
                            // Log unexpected message format for debugging
                            eprintln!("Unexpected WebSocket message (first 200 chars): {}",
                                     &text.chars().take(200).collect::<String>());
                            Some(Err(Error::Json(e)))
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    // Connection closed gracefully
                    Some(Err(Error::ConnectionClosed))
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // Ignore ping/pong frames (handled automatically)
                    None
                }
                Ok(Message::Binary(_)) => {
                    // Unexpected binary message
                    Some(Err(Error::WebSocket(
                        "Unexpected binary message".to_string(),
                    )))
                }
                Ok(Message::Frame(_)) => {
                    // Raw frame (shouldn't happen)
                    None
                }
                Err(e) => {
                    // WebSocket error
                    Some(Err(Error::WebSocket(e.to_string())))
                }
            }
        });

        Ok(Box::pin(stream))
    }
}

impl Default for MarketWsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MarketWsClient::new();
        assert_eq!(client.ws_url, MarketWsClient::DEFAULT_WS_URL);
    }

    #[test]
    fn test_client_with_custom_url() {
        let custom_url = "wss://custom.example.com/ws";
        let client = MarketWsClient::with_url(custom_url);
        assert_eq!(client.ws_url, custom_url);
    }
}
