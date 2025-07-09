//! Strike API SDK
//! Rust SDK for <https://strike.me/>
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use std::fmt;
use std::str::FromStr;

use anyhow::bail;
use rand::distr::Alphanumeric;
use rand::Rng;
use reqwest::{Client, IntoUrl, Url};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

mod error;
pub mod exchange;
pub(crate) mod hex;
pub mod invoice;
pub mod pay_ln;
pub mod webhooks;

pub use error::Error;
pub use error::StrikeErrorCode;
pub use exchange::*;
pub use invoice::*;
pub use pay_ln::*;

/// Strike
#[derive(Debug, Clone)]
pub struct Strike {
    api_key: String,
    base_url: Url,
    client: Client,
    webhook_secret: String,
}

/// Currency unit
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    /// USD
    USD,
    /// EURO
    EUR,
    /// Bitcoin
    BTC,
    /// Tether USD
    USDT,
    /// British Pound
    GBP,
    /// Australian Dollar
    AUD,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::USD => write!(f, "USD"),
            Self::EUR => write!(f, "EUR"),
            Self::BTC => write!(f, "BTC"),
            Self::USDT => write!(f, "USDT"),
            Self::GBP => write!(f, "GBP"),
            Self::AUD => write!(f, "AUD"),
        }
    }
}

/// Amount with unit
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Amount {
    /// Currency of amount
    pub currency: Currency,
    /// Value of amount
    #[serde(deserialize_with = "parse_f64_from_string")]
    pub amount: f64,
}

/// Amount with fee policy for payment requests
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAmount {
    /// Currency amount in decimal format
    #[serde(deserialize_with = "parse_f64_from_string")]
    pub amount: f64,
    /// Currency code
    pub currency: Currency,
    /// Should the fee be included in the amount or added on top of it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_policy: Option<FeePolicy>,
}

fn parse_f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

impl Amount {
    /// Amount from sats
    pub fn from_sats(amount: u64) -> Self {
        Self {
            currency: Currency::BTC,
            amount: amount as f64 / 100_000_000.0,
        }
    }

    /// Unit as sats
    pub fn to_sats(&self) -> anyhow::Result<u64> {
        match self.currency {
            Currency::BTC => Ok((self.amount * 100_000_000.0) as u64),
            _ => bail!("Unit cannot be converted to sats"),
        }
    }
}

/// Invoice state
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum InvoiceState {
    /// Payment Completed
    Completed,
    /// Invoice paid
    Paid,
    /// Invoice unpaid
    Unpaid,
    /// Invoice pending
    Pending,
    /// Payment failed
    Failed,
}

/// Payment result (obsolete, use InvoiceState instead)
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PaymentResult {
    /// Payment pending
    Pending,
    /// Payment successful
    Success,
    /// Payment failed
    Failure,
}

/// Fee policy for payments
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FeePolicy {
    /// Fee is included in the amount
    Inclusive,
    /// Fee is added on top of the amount
    Exclusive,
}

/// Conversion rate for quote
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ConversionRate {
    /// Amount
    #[serde(deserialize_with = "parse_f64_from_string")]
    pub amount: f64,
    /// Source Unit
    #[serde(rename = "sourceCurrency")]
    pub source_currency: Currency,
    /// Target Unit
    #[serde(rename = "targetCurrency")]
    pub target_currency: Currency,
}

/// Lightning network payment details
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightningPaymentDetails {
    /// The fee required by LN network, if any
    pub network_fee: Option<Amount>,
}

/// On-chain payment details
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnchainPaymentDetails {
    /// The COMPLETED onchain payment's transaction ID
    pub txn_id: Option<String>,
}

impl Strike {
    /// Create Strike client
    /// # Arguments
    /// * `api_key` - Strike api token
    /// * `url` - Optional Url of nodeless api
    ///
    /// # Example
    /// ```
    /// use strike_rs::Strike;
    /// let client = Strike::new("xxxxxxxxxxx", None).unwrap();
    /// ```
    pub fn new(api_key: &str, api_url: Option<String>) -> anyhow::Result<Self> {
        let base_url = match api_url {
            Some(url) => Url::from_str(&url)?,
            _ => Url::from_str("https://api.strike.me")?,
        };

        let client = reqwest::Client::builder().build()?;
        let secret: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        Ok(Self {
            api_key: api_key.to_string(),
            base_url,
            client,
            webhook_secret: secret,
        })
    }

    /// Handle API error response
    fn handle_api_error(&self, status: reqwest::StatusCode, text: &str) -> crate::Error {
        // Try to parse as Strike API error
        if let Ok(api_err) = serde_json::from_str::<crate::error::StrikeApiError>(text) {
            return crate::Error::ApiError(api_err);
        }

        // If parsing fails, create a generic error based on status code
        use crate::error::{StrikeApiError, StrikeApiErrorData, StrikeErrorCode};
        use std::collections::HashMap;

        let error_code = match status.as_u16() {
            400 => StrikeErrorCode::InvalidData,
            401 => StrikeErrorCode::Unauthorized,
            403 => StrikeErrorCode::Forbidden,
            404 => StrikeErrorCode::NotFound,
            409 => StrikeErrorCode::ProcessingConflict,
            422 => StrikeErrorCode::UnprocessableEntity,
            429 => StrikeErrorCode::RateLimitExceeded,
            500 => StrikeErrorCode::InternalServerError,
            502 => StrikeErrorCode::BadGateway,
            503 => StrikeErrorCode::ServiceUnavailable,
            504 => StrikeErrorCode::GatewayTimeout,
            _ => StrikeErrorCode::Unknown,
        };

        let api_error = StrikeApiError {
            trace_id: None,
            data: StrikeApiErrorData {
                status: status.as_u16(),
                code: error_code,
                message: if text.is_empty() {
                    format!(
                        "HTTP {}: {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("Unknown")
                    )
                } else {
                    text.to_string()
                },
                values: HashMap::new(),
                validation_errors: HashMap::new(),
                debug: None,
            },
        };

        crate::Error::ApiError(api_error)
    }

    async fn make_get<U>(&self, url: U) -> Result<Value, crate::Error>
    where
        U: IntoUrl,
    {
        let res = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("accept", "application/json")
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(self.handle_api_error(status, &text));
        }

        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    async fn make_post<U, T>(&self, url: U, data: Option<T>) -> Result<Value, crate::Error>
    where
        U: IntoUrl,
        T: Serialize,
    {
        let res = match data {
            Some(data) => {
                self.client
                    .post(url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .header("accept", "application/json")
                    .json(&data)
                    .send()
                    .await?
            }
            _ => {
                self.client
                    .post(url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Length", "0")
                    .header("accept", "application/json")
                    .send()
                    .await?
            }
        };

        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(self.handle_api_error(status, &text));
        }

        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    async fn make_patch<U>(&self, url: U) -> Result<Value, crate::Error>
    where
        U: IntoUrl,
    {
        let res = self
            .client
            .patch(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Length", "0")
            .header("accept", "application/json")
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(self.handle_api_error(status, &text));
        }

        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    async fn make_delete<U>(&self, url: U) -> Result<(), crate::Error>
    where
        U: IntoUrl,
    {
        let res = self
            .client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(self.handle_api_error(status, &text));
        }

        Ok(())
    }

    // NOTE: i was getting errors when making regular patch requests, so i made this one and it works,
    // but we shouldn't need it
    async fn make_patch_no_body<U>(&self, url: U) -> anyhow::Result<Value, crate::Error>
    where
        U: IntoUrl,
    {
        let res = self
            .client
            .patch(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Length", "0")
            .header("accept", "application/json")
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(self.handle_api_error(status, &text));
        }

        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    /*
    async fn make_put(&self, url: Url, data: Option<Value>) -> Result<Value> {
        let res = self
            .client
            .put(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("accept", "application/json")
            .json(&data)
            .send()
            .await?;
        let res = res.json::<Value>().await?;
        Ok(res)
    }

    */
}
