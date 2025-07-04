//! Handle currency exchange operations

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{ConversionRate, Currency, FeePolicy, Strike};

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
    pub source: ExchangeAmountResponse,
    /// Amount bought
    pub target: ExchangeAmountResponse,
    /// Service fee (if applicable)
    pub fee: Option<ExchangeAmountResponse>,
    /// Conversion rate used for the transaction
    pub conversion_rate: ConversionRate,
    /// State of the currency exchange quote
    pub state: ExchangeQuoteState,
    /// Timestamp at which the exchange was completed (if completed)
    pub completed: Option<String>,
}

/// Exchange amount response format
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExchangeAmountResponse {
    /// Currency amount in decimal format
    pub amount: String,
    /// Currency code
    pub currency: Currency,
}

impl Strike {
    /// Create currency exchange quote
    pub async fn create_currency_exchange_quote(
        &self,
        quote_request: CurrencyExchangeQuoteRequest,
    ) -> Result<CurrencyExchangeQuoteResponse> {
        let url = self.base_url.join("/v1/currency-exchange-quotes")?;

        let res = self
            .make_post(url, Some(serde_json::to_value(quote_request)?))
            .await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on currency exchange quote creation");
                log::error!("{}", res);
                bail!("Could not create currency exchange quote")
            }
        }
    }

    /// Execute currency exchange quote
    pub async fn execute_currency_exchange_quote(&self, quote_id: &str) -> Result<()> {
        let url = self.base_url.join(&format!(
            "/v1/currency-exchange-quotes/{}/execute",
            quote_id
        ))?;

        let res = self.make_patch_no_body(url).await?;
        if res.status() == 202 {
            Ok(())
        } else {
            let body = res.text().await?;
            log::error!("Unexpected response: {}", body);
            bail!("Could not execute currency exchange quote")
        }
    }

    /// Get currency exchange quote by ID
    pub async fn get_currency_exchange_quote(
        &self,
        quote_id: &str,
    ) -> Result<CurrencyExchangeQuoteResponse> {
        let url = self
            .base_url
            .join(&format!("/v1/currency-exchange-quotes/{}", quote_id))?;

        let res = self.make_get(url).await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on get currency exchange quote");
                log::error!("{}", res);
                bail!("Could not get currency exchange quote")
            }
        }
    }
}

impl CurrencyExchangeQuoteRequest {
    /// Create a new currency exchange quote request
    pub fn new(sell: Currency, buy: Currency, amount: String, currency: Currency) -> Self {
        Self {
            sell,
            buy,
            amount: ExchangeAmount {
                amount,
                currency,
                fee_policy: None,
            },
        }
    }

    /// Set fee policy for the exchange amount
    pub fn with_fee_policy(mut self, fee_policy: FeePolicy) -> Self {
        self.amount.fee_policy = Some(fee_policy);
        self
    }
}

impl ExchangeAmount {
    /// Create a new exchange amount
    pub fn new(amount: String, currency: Currency) -> Self {
        Self {
            amount,
            currency,
            fee_policy: None,
        }
    }

    /// Set fee policy
    pub fn with_fee_policy(mut self, fee_policy: FeePolicy) -> Self {
        self.fee_policy = Some(fee_policy);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_exchange_quote_request_creation() {
        let request = CurrencyExchangeQuoteRequest::new(
            Currency::USD,
            Currency::BTC,
            "5.00".to_string(),
            Currency::USD,
        );

        assert_eq!(request.sell, Currency::USD);
        assert_eq!(request.buy, Currency::BTC);
        assert_eq!(request.amount.amount, "5.00");
        assert_eq!(request.amount.currency, Currency::USD);
        assert_eq!(request.amount.fee_policy, None);
    }

    #[test]
    fn test_currency_exchange_quote_request_with_fee_policy() {
        let request = CurrencyExchangeQuoteRequest::new(
            Currency::USD,
            Currency::BTC,
            "5.00".to_string(),
            Currency::USD,
        )
        .with_fee_policy(FeePolicy::Inclusive);

        assert_eq!(request.amount.fee_policy, Some(FeePolicy::Inclusive));
    }

    #[test]
    fn test_exchange_amount_creation() {
        let amount = ExchangeAmount::new("10.50".to_string(), Currency::EUR);

        assert_eq!(amount.amount, "10.50");
        assert_eq!(amount.currency, Currency::EUR);
        assert_eq!(amount.fee_policy, None);
    }

    #[test]
    fn test_exchange_amount_with_fee_policy() {
        let amount = ExchangeAmount::new("10.50".to_string(), Currency::EUR)
            .with_fee_policy(FeePolicy::Exclusive);

        assert_eq!(amount.fee_policy, Some(FeePolicy::Exclusive));
    }
}
