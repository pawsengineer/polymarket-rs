use futures_util::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::sleep;

use crate::error::{Error, Result};

/// Configuration for reconnection behavior
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Initial delay before first reconnection attempt
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Maximum number of reconnection attempts (None = infinite)
    pub max_attempts: Option<u32>,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_attempts: None,
        }
    }
}

/// Exponential backoff calculator
#[derive(Debug, Clone)]
struct ExponentialBackoff {
    current_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl ExponentialBackoff {
    fn new(initial_delay: Duration, max_delay: Duration, multiplier: f64) -> Self {
        Self {
            current_delay: initial_delay,
            max_delay,
            multiplier,
        }
    }

    /// Get the next delay duration
    fn next_delay(&mut self) -> Duration {
        let delay = self.current_delay;
        self.current_delay = std::cmp::min(
            Duration::from_secs_f64(delay.as_secs_f64() * self.multiplier),
            self.max_delay,
        );
        delay
    }

    /// Reset the backoff to initial delay
    fn reset(&mut self) {
        self.current_delay = Duration::from_secs(1);
    }
}

/// State of the reconnecting stream
enum StreamState<S, Fut> {
    /// Currently connected and streaming
    Connected(S),
    /// Connection failed, waiting to reconnect
    Reconnecting {
        attempts: u32,
        delay: Duration,
    },
    /// Reconnection in progress
    Connecting {
        attempts: u32,
        future: Option<Pin<Box<Fut>>>,
    },
    /// Stream has been terminated
    Terminated,
}

/// A stream wrapper that automatically reconnects on disconnection
///
/// This wrapper provides resilient streaming by:
/// - Automatically reconnecting when the connection is lost
/// - Using exponential backoff between reconnection attempts
/// - Optionally limiting the number of reconnection attempts
///
/// # Example
///
/// ```no_run
/// use polymarket_rs::websocket::{MarketWsClient, ReconnectingStream, ReconnectConfig};
/// use futures_util::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = MarketWsClient::new();
///     let token_ids = vec!["token_id".to_string()];
///
///     let config = ReconnectConfig::default();
///     let mut stream = ReconnectingStream::new(
///         config,
///         move || {
///             let client = client.clone();
///             let token_ids = token_ids.clone();
///             async move { client.subscribe(token_ids).await }
///         },
///     );
///
///     while let Some(event) = stream.next().await {
///         println!("Event: {:?}", event?);
///     }
///
///     Ok(())
/// }
/// ```
pub struct ReconnectingStream<T, S, F, Fut>
where
    S: Stream<Item = Result<T>> + Unpin,
    F: Fn() -> Fut,
    Fut: Future<Output = Result<S>>,
{
    /// Function to create a new stream connection
    connect_fn: F,
    /// Current state of the stream
    state: StreamState<S, Fut>,
    /// Reconnection configuration
    config: ReconnectConfig,
    /// Exponential backoff calculator
    backoff: ExponentialBackoff,
    /// Sleep future for reconnection delay
    sleep_future: Option<Pin<Box<tokio::time::Sleep>>>,
}

impl<T, S, F, Fut> ReconnectingStream<T, S, F, Fut>
where
    S: Stream<Item = Result<T>> + Unpin,
    F: Fn() -> Fut,
    Fut: Future<Output = Result<S>>,
{
    /// Create a new reconnecting stream
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for reconnection behavior
    /// * `connect_fn` - Function that creates a new stream connection
    pub fn new(config: ReconnectConfig, connect_fn: F) -> Self {
        let backoff = ExponentialBackoff::new(
            config.initial_delay,
            config.max_delay,
            config.multiplier,
        );

        Self {
            connect_fn,
            state: StreamState::Connecting {
                attempts: 0,
                future: None,
            },
            config,
            backoff,
            sleep_future: None,
        }
    }

    /// Handle a disconnection and prepare for reconnection
    fn handle_disconnection(&mut self, attempts: u32) -> Poll<Option<Result<T>>> {
        // Check if we've exceeded max attempts
        if let Some(max) = self.config.max_attempts {
            if attempts >= max {
                self.state = StreamState::Terminated;
                return Poll::Ready(Some(Err(Error::ReconnectFailed {
                    attempts,
                    last_error: "Maximum reconnection attempts reached".to_string(),
                })));
            }
        }

        let delay = self.backoff.next_delay();
        self.state = StreamState::Reconnecting { attempts, delay };
        self.sleep_future = Some(Box::pin(sleep(delay)));
        Poll::Pending
    }
}

impl<T, S, F, Fut> Stream for ReconnectingStream<T, S, F, Fut>
where
    S: Stream<Item = Result<T>> + Unpin,
    F: Fn() -> Fut + Unpin,
    Fut: Future<Output = Result<S>>,
{
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match &mut self.state {
                StreamState::Connected(stream) => {
                    match Pin::new(stream).poll_next(cx) {
                        Poll::Ready(Some(Ok(item))) => {
                            // Successfully received an item, reset backoff
                            self.backoff.reset();
                            return Poll::Ready(Some(Ok(item)));
                        }
                        Poll::Ready(Some(Err(Error::ConnectionClosed))) => {
                            // Connection closed, prepare to reconnect
                            return self.handle_disconnection(1);
                        }
                        Poll::Ready(Some(Err(e))) => {
                            // Other error, pass through and prepare to reconnect
                            let _ = self.handle_disconnection(1);
                            return Poll::Ready(Some(Err(e)));
                        }
                        Poll::Ready(None) => {
                            // Stream ended, prepare to reconnect
                            return self.handle_disconnection(1);
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
                StreamState::Reconnecting { attempts, .. } => {
                    let attempts = *attempts;
                    // Wait for the sleep delay
                    if let Some(mut sleep_fut) = self.sleep_future.take() {
                        match Pin::new(&mut sleep_fut).poll(cx) {
                            Poll::Ready(()) => {
                                // Delay complete, start connecting
                                self.state = StreamState::Connecting {
                                    attempts,
                                    future: None,
                                };
                                continue;
                            }
                            Poll::Pending => {
                                self.sleep_future = Some(sleep_fut);
                                return Poll::Pending;
                            }
                        }
                    } else {
                        // No sleep future, start one
                        let delay = match &self.state {
                            StreamState::Reconnecting { delay, .. } => *delay,
                            _ => unreachable!(),
                        };
                        self.sleep_future = Some(Box::pin(sleep(delay)));
                        continue;
                    }
                }
                StreamState::Connecting { attempts, future } => {
                    let current_attempts = *attempts;
                    // Get or create the connection future
                    let mut boxed_fut = if let Some(fut) = future.take() {
                        fut
                    } else {
                        Box::pin((self.connect_fn)())
                    };

                    match boxed_fut.as_mut().poll(cx) {
                        Poll::Ready(Ok(stream)) => {
                            self.state = StreamState::Connected(stream);
                            self.backoff.reset();
                            continue;
                        }
                        Poll::Ready(Err(_e)) => {
                            // Connection failed, prepare to reconnect
                            // Increment attempts (or start at 1 if this is the first attempt)
                            let next_attempts = if current_attempts == 0 { 1 } else { current_attempts + 1 };
                            return self.handle_disconnection(next_attempts);
                        }
                        Poll::Pending => {
                            // Store the future for next poll
                            self.state = StreamState::Connecting {
                                attempts: current_attempts,
                                future: Some(boxed_fut),
                            };
                            return Poll::Pending;
                        }
                    }
                }
                StreamState::Terminated => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            2.0,
        );

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        assert_eq!(backoff.next_delay(), Duration::from_secs(8));
    }

    #[test]
    fn test_backoff_max() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_secs(1),
            Duration::from_secs(5),
            2.0,
        );

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));
        assert_eq!(backoff.next_delay(), Duration::from_secs(4));
        assert_eq!(backoff.next_delay(), Duration::from_secs(5)); // capped
        assert_eq!(backoff.next_delay(), Duration::from_secs(5)); // still capped
    }

    #[test]
    fn test_backoff_reset() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            2.0,
        );

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
        assert_eq!(backoff.next_delay(), Duration::from_secs(2));

        backoff.reset();

        assert_eq!(backoff.next_delay(), Duration::from_secs(1));
    }
}
