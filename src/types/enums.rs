use serde::{Deserialize, Serialize};

/// Asset type for balance and allowance operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetType {
    Collateral,
    Conditional,
}

/// Order side (BUY or SELL)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    #[default]
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl Side {
    /// Convert side to numeric value (0 for BUY, 1 for SELL)
    pub fn to_u8(self) -> u8 {
        match self {
            Side::Buy => 0,
            Side::Sell => 1,
        }
    }

    /// Convert side to string ("BUY" or "SELL")
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }

    /// Create side from numeric value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Side::Buy),
            1 => Some(Side::Sell),
            _ => None,
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good till canceled
    #[serde(rename = "GTC")]
    Gtc,
    /// Fill or kill
    #[serde(rename = "FOK")]
    Fok,
    /// Good till date
    #[serde(rename = "GTD")]
    Gtd,
}

/// Signature type for orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    /// EOA (Externally Owned Account) signature
    #[serde(rename = "0")]
    Eoa = 0,
    /// Poly Proxy wallet signature
    #[serde(rename = "1")]
    PolyProxy = 1,
    /// Poly Gnosis Safe signature
    #[serde(rename = "2")]
    PolyGnosisSafe = 2,
}

impl SignatureType {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SignatureType::Eoa),
            1 => Some(SignatureType::PolyProxy),
            2 => Some(SignatureType::PolyGnosisSafe),
            _ => None,
        }
    }
}

/// Market status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    Active,
    Closed,
    Archived,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    Live,
    Matched,
    Canceled,
    Expired,
}

/// Notification type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    OrderMatched,
    OrderCanceled,
    OrderExpired,
    BalanceUpdate,
    Other(String),
}

/// Activity type
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActivityType {
    #[default]
    Trade,
    Yield,
    Reward,
    Split,
    Merge,
    Conversion,
    Redeem,
}
