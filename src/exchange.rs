//! Handle currency exchange operations

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Amount, ConversionRate, Currency, FeePolicy, Strike};

/// Currency exchange quote request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CurrencyExchangeQuoteRequest {
    /// Currency to sell
    pub sell: Currency,
    /// Currency to buy
    pub buy: Currency,
    /// Amount and currency to convert
    pub amount: ExchangeAmount,
}

/// Amount with optional fee policy for exchange requests
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeAmount {
    /// Currency amount in decimal format
    pub amount: String,
    /// Currency code
    pub currency: Currency,
    /// Should the fee be included in the amount or added on top of it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_policy: Option<FeePolicy>,
}

/// Currency exchange quote state
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ExchangeQuoteState {
    /// New quote
    New,
    /// Quote pending execution
    Pending,
    /// Quote completed successfully
    Completed,
    /// Quote failed
    Failed,
}

/// Currency exchange quote response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyExchangeQuoteResponse {
    /// ID of the exchange quote
    pub id: String,
    /// Date and time when the exchange quote was created
    pub created: String,
    /// Date and time when the exchange quote expires
    pub valid_until: String,
    /// Amount sold
    pub source: Amount,
    /// Amount bought
    pub target: Amount,
    /// Service fee (if applicable)
    pub fee: Option<Amount>,
    /// Conversion rate used for the transaction
    pub conversion_rate: ConversionRate,
    /// State of the currency exchange quote
    pub state: ExchangeQuoteState,
    /// Timestamp at which the exchange was completed (if completed)
    pub completed: Option<String>,
}

impl Strike {
    /// Create currency exchange quote
    pub async fn create_currency_exchange_quote(
        &self,
        quote_request: CurrencyExchangeQuoteRequest,
    ) -> Result<CurrencyExchangeQuoteResponse, crate::Error> {
        let url = self.base_url.join("/v1/currency-exchange-quotes")?;

        let res = self
            .make_post(url, Some(serde_json::to_value(quote_request)?))
            .await?;

        let quote: CurrencyExchangeQuoteResponse = serde_json::from_value(res.clone())?;
        Ok(quote)
    }

    /// Execute currency exchange quote
    pub async fn execute_currency_exchange_quote(
        &self,
        quote_id: &str,
    ) -> Result<(), crate::Error> {
        let url = self.base_url.join(&format!(
            "/v1/currency-exchange-quotes/{}/execute",
            quote_id
        ))?;

        self.make_patch_no_body(url).await?;
        Ok(())
    }

    /// Get currency exchange quote by ID
    pub async fn get_currency_exchange_quote(
        &self,
        quote_id: &str,
    ) -> Result<CurrencyExchangeQuoteResponse, crate::Error> {
        let url = self
            .base_url
            .join(&format!("/v1/currency-exchange-quotes/{}", quote_id))?;

        let res = self.make_get(url).await?;

        let quote: CurrencyExchangeQuoteResponse = serde_json::from_value(res.clone())?;
        Ok(quote)
    }
}
