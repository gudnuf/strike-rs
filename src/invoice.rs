//! Handle invoice creation

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{Amount, ConversionRate, InvoiceState, Strike};

/// Invoice Request
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceRequest {
    /// Correlation ID
    pub correlation_id: Option<String>,
    /// Invoice description
    pub description: Option<String>,
    /// Invoice [`Amount`]
    pub amount: Amount,
}

/// Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceResponse {
    /// Invoice ID
    pub invoice_id: String,
    /// Invoice [`Amount`]
    pub amount: Amount,
    /// Invoice State
    pub state: InvoiceState,
    /// Created timestamp
    pub created: String,
    /// Invoice Description
    pub description: Option<String>,
    /// Isser ID
    pub issuer_id: String,
    /// Receiver ID
    pub receiver_id: String,
}

/// Invoice Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceQuoteResponse {
    /// Invoice Quote ID
    pub quote_id: String,
    /// Invoice description
    pub description: Option<String>,
    /// Bolt11 invoice
    pub ln_invoice: String,
    /// Onchain Address
    pub onchain_address: Option<String>,
    /// Expiration of quote
    pub expiration: String,
    /// Experition in secs
    pub expiration_in_sec: u64,
    /// Source Amount
    pub source_amount: Amount,
    /// Target Amount
    pub target_amount: Amount,
    /// Conversion Rate
    pub conversion_rate: ConversionRate,
}

/// Invoice List Item for paginated results
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceListItem {
    /// Invoice ID
    pub invoice_id: String,
    /// Invoice [`Amount`]
    pub amount: Amount,
    /// Invoice State
    pub state: InvoiceState,
    /// Created timestamp
    pub created: String,
    /// Correlation ID
    pub correlation_id: Option<String>,
    /// Invoice Description
    pub description: Option<String>,
    /// Issuer ID
    pub issuer_id: String,
    /// Receiver ID
    pub receiver_id: String,
    /// Payer ID (present only when invoice was created for the dedicated payer)
    pub payer_id: Option<String>,
}

/// Invoice List Response
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InvoiceListResponse {
    /// The page items
    pub items: Vec<InvoiceListItem>,
    /// Total number of records
    pub count: i64,
}

/// Query parameters for getting invoices
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InvoiceQueryParams {
    /// Filter the results using OData syntax
    pub filter: Option<String>,
    /// Order the results using OData syntax
    pub orderby: Option<String>,
    /// Skip the specified number of entries
    pub skip: Option<i32>,
    /// Get the top X number of records (max 100)
    pub top: Option<i32>,
}

impl InvoiceQueryParams {
    /// Create new query parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set filter parameter
    pub fn filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set orderby parameter
    pub fn orderby(mut self, orderby: String) -> Self {
        self.orderby = Some(orderby);
        self
    }

    /// Set skip parameter
    pub fn skip(mut self, skip: i32) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Set top parameter (max 100)
    pub fn top(mut self, top: i32) -> Self {
        self.top = Some(top.min(100));
        self
    }

    /// Convert to query string
    fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(filter) = &self.filter {
            params.push(format!("$filter={}", urlencoding::encode(filter)));
        }

        if let Some(orderby) = &self.orderby {
            params.push(format!("$orderby={}", urlencoding::encode(orderby)));
        }

        if let Some(skip) = self.skip {
            params.push(format!("$skip={}", skip));
        }

        if let Some(top) = self.top {
            params.push(format!("$top={}", top));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

impl Strike {
    /// Create Invoice
    pub async fn create_invoice(&self, invoice_request: InvoiceRequest) -> Result<InvoiceResponse> {
        let url = self.base_url.join("/v1/invoices")?;

        let res = self
            .make_post(url, Some(serde_json::to_value(invoice_request)?))
            .await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on invoice creation");
                log::error!("{}", res);
                bail!("Could not create invoice")
            }
        }
    }

    /// Find incoming invoice
    pub async fn get_incoming_invoice(&self, invoice_id: &str) -> Result<InvoiceResponse> {
        let url = self.base_url.join("/v1/invoices/")?.join(invoice_id)?;

        let res = self.make_get(url).await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on find invoice");
                log::error!("{}", res);
                bail!("Could not find invoice")
            }
        }
    }

    /// Get invoices with filtering and pagination
    pub async fn get_invoices(
        &self,
        params: Option<InvoiceQueryParams>,
    ) -> Result<InvoiceListResponse> {
        let query_string = params.unwrap_or_default().to_query_string();
        let url_string = format!("/v1/invoices{}", query_string);
        let url = self.base_url.join(&url_string)?;

        let res = self.make_get(url).await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on get invoices");
                log::error!("{}", res);
                bail!("Could not get invoices")
            }
        }
    }

    /// Invoice quote
    pub async fn invoice_quote(&self, invoice_id: &str) -> Result<InvoiceQuoteResponse> {
        let url = self
            .base_url
            .join(&format!("/v1/invoices/{invoice_id}/quote"))?;

        let res = self.make_post(url, None::<String>).await?;

        match serde_json::from_value(res.clone()) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on invoice quote");
                log::error!("{}", res);
                bail!("Could get invoice quote")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_query_params_empty() {
        let params = InvoiceQueryParams::new();
        assert_eq!(params.to_query_string(), "");
    }

    #[test]
    fn test_invoice_query_params_single() {
        let params = InvoiceQueryParams::new().filter("state eq 'PAID'".to_string());
        assert_eq!(params.to_query_string(), "?$filter=state%20eq%20%27PAID%27");
    }

    #[test]
    fn test_invoice_query_params_multiple() {
        let params = InvoiceQueryParams::new()
            .filter("state eq 'PAID'".to_string())
            .orderby("created desc".to_string())
            .top(10)
            .skip(5);
        let query = params.to_query_string();
        assert!(query.starts_with("?"));
        assert!(query.contains("$filter=state%20eq%20%27PAID%27"));
        assert!(query.contains("$orderby=created%20desc"));
        assert!(query.contains("$top=10"));
        assert!(query.contains("$skip=5"));
    }

    #[test]
    fn test_invoice_query_params_top_limit() {
        let params = InvoiceQueryParams::new().top(150); // Should be limited to 100
        assert_eq!(params.top, Some(100));
    }
}
