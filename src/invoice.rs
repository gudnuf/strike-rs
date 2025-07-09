//! Handle invoice creation

use anyhow::Result;
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

/// Supported filter operations for OData queries.
#[derive(Clone, Debug, PartialEq)]
pub enum FilterOp {
    /// Equal to
    Eq,
    /// Not equal to
    Ne,
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Greater than or equal to
    Ge,
    /// Less than or equal to
    Le,
    // Add more as needed
}

/// Represents a single OData filter condition for invoice queries.
#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    /// The field to filter on (as specified in the API spec)
    pub field: &'static str,
    /// The filter operation (e.g., Eq, Ne, Gt, etc.)
    pub op: FilterOp,
    /// The value to compare against
    pub value: String,
}

impl Filter {
    /// Create an equality filter (field eq value)
    pub fn eq(field: &'static str, value: impl ToString) -> Self {
        Self {
            field,
            op: FilterOp::Eq,
            value: value.to_string(),
        }
    }
    /// Create a not-equal filter (field ne value)
    pub fn ne(field: &'static str, value: impl ToString) -> Self {
        Self {
            field,
            op: FilterOp::Ne,
            value: value.to_string(),
        }
    }
    /// Create a greater-than filter (field gt value)
    pub fn gt(field: &'static str, value: impl ToString) -> Self {
        Self {
            field,
            op: FilterOp::Gt,
            value: value.to_string(),
        }
    }
    /// Create a less-than filter (field lt value)
    pub fn lt(field: &'static str, value: impl ToString) -> Self {
        Self {
            field,
            op: FilterOp::Lt,
            value: value.to_string(),
        }
    }
    /// Create a greater-than-or-equal filter (field ge value)
    pub fn ge(field: &'static str, value: impl ToString) -> Self {
        Self {
            field,
            op: FilterOp::Ge,
            value: value.to_string(),
        }
    }
    /// Create a less-than-or-equal filter (field le value)
    pub fn le(field: &'static str, value: impl ToString) -> Self {
        Self {
            field,
            op: FilterOp::Le,
            value: value.to_string(),
        }
    }
    /// Convert the filter to an OData filter string (e.g., "field eq 'value'")
    pub fn to_string(&self) -> String {
        let op_str = match self.op {
            FilterOp::Eq => "eq",
            FilterOp::Ne => "ne",
            FilterOp::Gt => "gt",
            FilterOp::Lt => "lt",
            FilterOp::Ge => "ge",
            FilterOp::Le => "le",
        };
        format!("{} {} '{}'", self.field, op_str, self.value)
    }
}

/// Query parameters for getting invoices, supporting flexible OData-style filtering and pagination.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InvoiceQueryParams {
    /// OData filters as a list of Filter objects
    pub filters: Vec<Filter>,
    /// Order the results using OData syntax
    pub orderby: Option<String>,
    /// Skip the specified number of entries
    pub skip: Option<i32>,
    /// Get the top X number of records (max 100)
    pub top: Option<i32>,
}

impl InvoiceQueryParams {
    /// Create new query parameters with no filters or pagination.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a filter condition to the query parameters.
    ///
    /// Filters are combined with logical AND in the resulting OData query.
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set the orderby parameter (OData syntax).
    pub fn orderby(mut self, orderby: impl Into<String>) -> Self {
        self.orderby = Some(orderby.into());
        self
    }

    /// Set the number of records to skip.
    pub fn skip(mut self, skip: i32) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Set the maximum number of records to return (capped at 100).
    pub fn top(mut self, top: i32) -> Self {
        self.top = Some(top.min(100));
        self
    }

    /// Convert the query parameters to a query string suitable for an HTTP request.
    ///
    /// Filters are joined with 'and' and URL-encoded. Other parameters are appended as needed.
    fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if !self.filters.is_empty() {
            let filter_str = self
                .filters
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(" and ");
            params.push(format!("$filter={}", urlencoding::encode(&filter_str)));
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
    pub async fn create_invoice(
        &self,
        invoice_request: InvoiceRequest,
    ) -> Result<InvoiceResponse, crate::Error> {
        let url = self.base_url.join("/v1/invoices")?;

        let res = self
            .make_post(url, Some(serde_json::to_value(invoice_request)?))
            .await?;

        let invoice: InvoiceResponse = serde_json::from_value(res.clone())?;
        Ok(invoice)
    }

    /// Find incoming invoice
    pub async fn get_incoming_invoice(
        &self,
        invoice_id: &str,
    ) -> Result<InvoiceResponse, crate::Error> {
        let url = self.base_url.join("/v1/invoices/")?.join(invoice_id)?;

        let res = self.make_get(url).await?;

        let invoice: InvoiceResponse = serde_json::from_value(res.clone())?;
        Ok(invoice)
    }

    /// Get invoices with filtering and pagination
    pub async fn get_invoices(
        &self,
        params: Option<InvoiceQueryParams>,
    ) -> Result<InvoiceListResponse, crate::Error> {
        let query_string = params.unwrap_or_default().to_query_string();
        let url_string = format!("/v1/invoices{}", query_string);
        let url = self.base_url.join(&url_string)?;

        let res = self.make_get(url).await?;

        let list: InvoiceListResponse = serde_json::from_value(res.clone())?;
        Ok(list)
    }

    /// Invoice quote
    pub async fn invoice_quote(
        &self,
        invoice_id: &str,
    ) -> Result<InvoiceQuoteResponse, crate::Error> {
        let url = self
            .base_url
            .join(&format!("/v1/invoices/{invoice_id}/quote"))?;

        let res = self.make_post(url, None::<String>).await?;

        let quote: InvoiceQuoteResponse = serde_json::from_value(res.clone())?;
        Ok(quote)
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
        let params = InvoiceQueryParams::new().filter(Filter::eq("state", "PAID"));
        assert_eq!(params.to_query_string(), "?$filter=state%20eq%20%27PAID%27");
    }

    #[test]
    fn test_invoice_query_params_multiple() {
        let params = InvoiceQueryParams::new()
            .filter(Filter::eq("state", "PAID"))
            .filter(Filter::eq("correlationId", "foo"))
            .orderby("created desc")
            .top(10)
            .skip(5);
        let query = params.to_query_string();
        assert!(query.starts_with("?"));
        assert!(
            query
                .contains("$filter=state%20eq%20%27PAID%27%20and%20correlationId%20eq%20%27foo%27")
                || query.contains(
                    "$filter=correlationId%20eq%20%27foo%27%20and%20state%20eq%20%27PAID%27"
                )
        );
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
