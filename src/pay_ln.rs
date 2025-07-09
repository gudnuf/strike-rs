//! Pay Ln

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    Amount, ConversionRate, Currency, InvoiceState, LightningPaymentDetails, OnchainPaymentDetails,
    Strike,
};

/// Pay Invoice Request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayInvoiceQuoteRequest {
    /// Bolt11 Invoice to pay
    pub ln_invoice: String,
    /// Currency to send from
    pub source_currency: Currency,
}

/// Pay Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayInvoiceQuoteResponse {
    /// Payment quote Id
    pub payment_quote_id: String,
    /// Description
    pub description: Option<String>,
    /// Quote valid till
    pub valid_until: Option<String>,
    /// Conversion quote
    pub conversion_rate: Option<ConversionRate>,
    /// The amount that the receiver will receive in sender’s currency
    pub amount: Amount,
    /// Network fee
    pub lightning_network_fee: Option<Amount>,
    /// The total of all fees
    pub total_fee: Option<Amount>,
    /// The amount that the sender will spend
    pub total_amount: Amount,
    /// Reward that the sender might receive, if applicable
    pub reward: Option<Amount>,
}

/// Pay Quote Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoicePaymentResponse {
    /// Payment id
    pub payment_id: String,
    /// Status of the payment execution
    pub state: InvoiceState,
    /// The timestamp of the payment completion
    pub completed: Option<String>,
    /// Conversion quote
    pub conversion_rate: Option<ConversionRate>,
    /// The amount that the receiver will receive in sender's currency
    pub amount: Amount,
    /// The total of all fees
    pub total_fee: Option<Amount>,
    /// The fee required by LN network. Only applicable for LN payments
    pub lightning_network_fee: Option<Amount>,
    /// The amount that the sender will spend
    pub total_amount: Amount,
    /// Reward that the sender might receive, if applicable
    pub reward: Option<Amount>,
    /// Details about the payment if it was made through the Lightning Network
    pub lightning: Option<LightningPaymentDetails>,
    /// Details about the payment if it was made on chain
    pub onchain: Option<OnchainPaymentDetails>,
}

impl Strike {
    /// Create Payment Quote
    pub async fn payment_quote(
        &self,
        quote_request: PayInvoiceQuoteRequest,
    ) -> Result<PayInvoiceQuoteResponse, crate::Error> {
        let url = self.base_url.join("/v1/payment-quotes/lightning")?;

        let res = self
            .make_post(url, Some(serde_json::to_value(quote_request)?))
            .await?;

        let quote: PayInvoiceQuoteResponse = serde_json::from_value(res.clone())?;
        Ok(quote)
    }

    /// Execute quote to pay invoice
    pub async fn pay_quote(
        &self,
        payment_quote_id: &str,
    ) -> Result<InvoicePaymentResponse, crate::Error> {
        let url = self
            .base_url
            .join(&format!("/v1/payment-quotes/{payment_quote_id}/execute"))?;

        let res = self.make_patch(url).await?;

        let payment: InvoicePaymentResponse = serde_json::from_value(res.clone())?;
        Ok(payment)
    }

    /// Get outgoing payment by payment id
    pub async fn get_outgoing_payment(
        &self,
        payment_id: &str,
    ) -> Result<InvoicePaymentResponse, crate::Error> {
        let url = self.base_url.join(&format!("/v1/payments/{payment_id}"))?;

        let res = self.make_get(url).await?;

        let payment: InvoicePaymentResponse = serde_json::from_value(res.clone())?;
        Ok(payment)
    }
}
