use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

/// Deserialize a number from either a string or number
pub fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}

/// Deserialize Decimal from JSON number (f64/int) or string
pub fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr {
        Str(String),
        F64(f64),
        U64(u64),
        I64(i64),
    }

    match Repr::deserialize(deserializer)? {
        Repr::Str(s) => Decimal::from_str(&s).map_err(serde::de::Error::custom),
        Repr::F64(f) => {
            Decimal::from_f64(f).ok_or_else(|| serde::de::Error::custom("invalid f64 for Decimal"))
        }
        Repr::U64(u) => Ok(Decimal::from(u)),
        Repr::I64(i) => Ok(Decimal::from(i)),
    }
}

/// Deserialize Option<DateTime<Utc>> from an optional RFC3339 string
pub fn deserialize_optional_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let dt = DateTime::parse_from_rfc3339(&s)
                .map_err(serde::de::Error::custom)?
                .with_timezone(&Utc);
            Ok(Some(dt))
        }
        None => Ok(None),
    }
}
