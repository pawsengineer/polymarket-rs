use futures_util::{SinkExt, Stream, StreamExt};
use std::pin::Pin;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::error::{Error, Result};
use crate::types::{ApiCreds, UserAuthentication, UserWsEvent};

/// WebSocket client for streaming authenticated user events
///
/// This client connects to the Polymarket CLOB user WebSocket endpoint and streams
/// real-time updates about the user's trades and orders.
///
/// # Connection Management
///
/// The Polymarket WebSocket server will disconnect idle connections after 1-2 minutes.
/// For production use, it's recommended to use [`ReconnectingStream`](crate::websocket::ReconnectingStream)
/// to automatically handle disconnections and reconnect with exponential backoff.
///
/// # Example with Auto-Reconnect
///
/// ```no_run
/// use polymarket_rs::websocket::{UserWsClient, ReconnectConfig, ReconnectingStream};
/// use polymarket_rs::types::ApiCreds;
/// use futures_util::StreamExt;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let creds = ApiCreds::new(
///         "api_key".to_string(),
///         "api_secret".to_string(),
///         "api_passphrase".to_string(),
///     );
///
///     let client = UserWsClient::new();
///
///     let config = ReconnectConfig {
///         initial_delay: Duration::from_secs(1),
///         max_delay: Duration::from_secs(30),
///         multiplier: 2.0,
///         max_attempts: None,
///     };
///
///     let creds_clone = creds.clone();
///     let mut stream = ReconnectingStream::new(config, move || {
///         let client = client.clone();
///         let creds = creds_clone.clone();
///         async move { client.subscribe_with_creds(&creds).await }
///     });
///
///     while let Some(event) = stream.next().await {
///         match event {
///             Ok(evt) => println!("Event: {:?}", evt),
///             Err(_) => continue, // Will auto-reconnect
///         }
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UserWsClient {
    ws_url: String,
}

impl UserWsClient {
    /// Default WebSocket URL for user events
    const DEFAULT_WS_URL: &'static str = "wss://ws-subscriptions-clob.polymarket.com/ws/user";

    /// Create a new user WebSocket client with the default endpoint
    pub fn new() -> Self {
        Self {
            ws_url: Self::DEFAULT_WS_URL.to_string(),
        }
    }

    /// Create a new user WebSocket client with a custom endpoint
    pub fn with_url(ws_url: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
        }
    }

    /// Subscribe to user events with API credentials
    ///
    /// Returns a stream of [`UserWsEvent`] items. The stream will yield events as they
    /// are received from the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `creds` - API credentials (api_key, secret, passphrase)
    ///
    /// # Events
    ///
    /// The stream will yield two types of events:
    /// - [`UserWsEvent::Trade`]: Trade execution updates
    /// - [`UserWsEvent::Order`]: Order status updates
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The WebSocket connection fails
    /// - The authentication message cannot be sent
    /// - Authentication fails (server will close the connection)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polymarket_rs::websocket::UserWsClient;
    /// # use polymarket_rs::types::ApiCreds;
    /// # use futures_util::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let creds = ApiCreds::new(
    ///     "your_api_key".to_string(),
    ///     "your_api_secret".to_string(),
    ///     "your_api_passphrase".to_string(),
    /// );
    ///
    /// let client = UserWsClient::new();
    /// let mut stream = client.subscribe_with_creds(&creds).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     println!("Event: {:?}", event?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_with_creds(
        &self,
        creds: &ApiCreds,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<UserWsEvent>> + Send>>> {
        self.subscribe(
            creds.api_key.clone(),
            creds.secret.clone(),
            creds.passphrase.clone(),
        )
        .await
    }

    /// Subscribe to user events with authentication
    ///
    /// Returns a stream of [`UserWsEvent`] items. The stream will yield events as they
    /// are received from the WebSocket connection.
    ///
    /// # Arguments
    ///
    /// * `api_key` - API key for authentication
    /// * `api_secret` - API secret for authentication
    /// * `api_passphrase` - API passphrase for authentication
    ///
    /// # Events
    ///
    /// The stream will yield two types of events:
    /// - [`UserWsEvent::Trade`]: Trade execution updates
    /// - [`UserWsEvent::Order`]: Order status updates
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The WebSocket connection fails
    /// - The authentication message cannot be sent
    /// - Authentication fails (server will close the connection)
    pub async fn subscribe(
        &self,
        api_key: String,
        api_secret: String,
        api_passphrase: String,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<UserWsEvent>> + Send>>> {
        // Connect to the WebSocket endpoint
        let (ws_stream, _) = connect_async(&self.ws_url).await?;

        let (mut write, read) = ws_stream.split();

        // Create authentication message
        let auth = UserAuthentication::new(api_key, api_secret, api_passphrase);

        let auth_msg = serde_json::to_string(&auth)?;

        // Send authentication message
        write
            .send(Message::Text(auth_msg))
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;

        // Return stream that parses events
        let stream = read.filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(text)) => {
                    // The server can send either a single object or an array
                    // Try to parse as array first
                    if let Ok(events) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                        // Got an array, take the first event
                        if let Some(first) = events.first() {
                            match serde_json::from_value::<UserWsEvent>(first.clone()) {
                                Ok(event) => return Some(Ok(event)),
                                Err(e) => return Some(Err(Error::Json(e))),
                            }
                        } else {
                            // Empty array, ignore
                            return None;
                        }
                    }

                    // Try parsing as single object
                    match serde_json::from_str::<UserWsEvent>(&text) {
                        Ok(event) => Some(Ok(event)),
                        Err(e) => Some(Err(Error::Json(e))),
                    }
                }
                Ok(Message::Close(close_frame)) => {
                    // Connection closed - may indicate auth failure
                    if let Some(frame) = close_frame {
                        Some(Err(Error::WebSocket(format!(
                            "Connection closed: code={}, reason={}",
                            frame.code, frame.reason
                        ))))
                    } else {
                        Some(Err(Error::ConnectionClosed))
                    }
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

impl Default for UserWsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = UserWsClient::new();
        assert_eq!(client.ws_url, UserWsClient::DEFAULT_WS_URL);
    }

    #[test]
    fn test_client_with_custom_url() {
        let custom_url = "wss://custom.example.com/ws/user";
        let client = UserWsClient::with_url(custom_url);
        assert_eq!(client.ws_url, custom_url);
    }
}
