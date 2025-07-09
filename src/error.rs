use std::collections::HashMap;
use thiserror::Error;

/// Strike API error response
///
/// For errors (non 2xx responses), Strike includes extra information about what went wrong
/// encoded in the response as JSON. Error responses conform to a standard schema with
/// a trace ID for debugging and detailed error information.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeApiError {
    /// Unique identifier which represents this request in trace logs
    pub trace_id: Option<String>,
    /// Error details
    pub data: StrikeApiErrorData,
}

/// Strike API error data containing detailed error information
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrikeApiErrorData {
    /// HTTP status code - matches the status code of the response
    pub status: u16,
    /// Error code - can contain any error code from the Strike API error codes
    pub code: StrikeErrorCode,
    /// Error message (for developer reference only) - should not be shown to the user
    pub message: String,
    /// Additional error properties - contains more details about the error
    #[serde(default)]
    pub values: HashMap<String, serde_json::Value>,
    /// Validation errors (present when code is INVALID_DATA)
    /// Key is the field name, value is an array of validation errors for that field
    #[serde(default)]
    pub validation_errors: HashMap<String, Vec<ValidationError>>,
    /// Debug information (only in development mode)
    pub debug: Option<DebugInfo>,
}

/// Validation error details for specific field validation failures
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ValidationError {
    /// Validation error code
    pub code: ValidationErrorCode,
    /// Error message (for developer reference only) - should not be shown to the user
    pub message: String,
    /// Additional validation error properties - contains more details about the validation error
    #[serde(default)]
    pub values: HashMap<String, serde_json::Value>,
}

/// Debug information included in error responses (development mode only)
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct DebugInfo {
    /// Full debug info with stack trace
    pub full: Option<String>,
    /// Additional debug info
    pub body: Option<String>,
}

/// Strike API error codes
///
/// These codes indicate the specific type of error that occurred.
/// Codes in the 2xx range indicate success, 4xx range indicate client errors,
/// and 5xx range indicate server errors.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrikeErrorCode {
    /// Resource not found (404)
    NotFound,
    /// The server has encountered an unexpected problem (500)
    InternalServerError,
    /// Bad gateway (502)
    BadGateway,
    /// Maintenance mode is active (503)
    MaintenanceMode,
    /// Service is temporarily unavailable (503)
    ServiceUnavailable,
    /// Gateway timeout (504)
    GatewayTimeout,
    /// Rate limit has been exceeded (429)
    RateLimitExceeded,
    /// Too many attempts. Try again in a few minutes (429)
    TooManyAttempts,
    /// Could not process the request at this time. Try again later (409)
    ProcessingConflict,
    /// Invalid or unspecified identity (401)
    Unauthorized,
    /// Insufficient permissions (403)
    Forbidden,
    /// One or more validation errors occurred (400)
    InvalidData,
    /// Invalid OData query (400)
    InvalidDataQuery,
    /// Unprocessable entity (422)
    UnprocessableEntity,
    /// Account is not yet ready. Please retry later (425)
    AccountNotReady,
    /// Invoice has already been paid (422)
    InvalidStateForInvoicePaid,
    /// Invoice has been reversed (422)
    InvalidStateForInvoiceReversed,
    /// Invoice has been cancelled (422)
    InvalidStateForInvoiceCancelled,
    /// Recipient is currently unable to receive payments (422)
    InvalidRecipient,
    /// Invoice payment is currently being processed (422)
    ProcessingPayment,
    /// Invoice with correlation id already exists (409)
    DuplicateInvoice,
    /// Self-payments are not allowed (422)
    SelfPaymentNotAllowed,
    /// User does not support the given currency (422)
    UserCurrencyUnavailable,
    /// Lightning network is currently unavailable (503)
    LnUnavailable,
    /// Exchange rate is unavailable for the given currency pair (422)
    ExchangeRateNotAvailable,
    /// Insufficient funds (422)
    BalanceTooLow,
    /// Amount must be greater than zero (422)
    InvalidAmount,
    /// The amount can't be above the limit (422)
    AmountTooHigh,
    /// The amount can't be below the limit (422)
    AmountTooLow,
    /// The payment method is not supported (422)
    UnsupportedPaymentMethod,
    /// Invalid payment method (422)
    InvalidPaymentMethod,
    /// The requested currency is not supported (422)
    CurrencyUnsupported,
    /// Account verification failed (422)
    PlaidLinkingFailed,
    /// Payment method must be in the READY state (422)
    PaymentMethodNotReady,
    /// Payout originator must be in the APPROVED state (422)
    PayoutOriginatorNotApproved,
    /// Payout has already been initiated (422)
    PayoutAlreadyInitiated,
    /// Lightning invoice has expired (422)
    InvalidStateForInvoiceExpired,
    /// Payment was already processed (422)
    PaymentProcessed,
    /// Invalid lightning invoice (422)
    InvalidLnInvoice,
    /// Lightning invoice has already been processed (422)
    LnInvoiceProcessed,
    /// Invalid btc address (422)
    InvalidBitcoinAddress,
    /// Unable to find a lightning payment route (422)
    LnRouteNotFound,
    /// The payment quote has expired (422)
    PaymentQuoteExpired,
    /// A payment quote for the specified idempotency key already exists (422)
    DuplicatePaymentQuote,
    /// Number of transactions limit exceeded (422)
    TooManyTransactions,
    /// A currency exchange quote for the specified idempotency key already exists (422)
    DuplicateCurrencyExchangeQuote,
    /// The currency exchange quote has already been processed (422)
    CurrencyExchangeQuoteProcessed,
    /// The currency exchange quote has expired (422)
    CurrencyExchangeQuoteExpired,
    /// The currency exchange pair is invalid (422)
    CurrencyExchangePairNotSupported,
    /// The exchanged amount is too low (422)
    CurrencyExchangeAmountTooLow,
    /// The number of allowed deposits has been exceeded (422)
    DepositLimitExceeded,
    /// A deposit for the specified idempotency key already exists (422)
    DuplicateDeposit,
    /// Unknown error code (for forward compatibility)
    #[serde(other)]
    Unknown,
}

/// Validation error codes
///
/// These codes indicate specific validation failures when the main error code is INVALID_DATA.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationErrorCode {
    /// General invalid data error
    InvalidData,
    /// The field is required
    InvalidDataRequired,
    /// The field must be a value with a specific length
    InvalidDataLength,
    /// The field must be a value with a minimum length
    InvalidDataMinlength,
    /// The field must be a value with a maximum length
    InvalidDataMaxlength,
    /// The field contains invalid value
    InvalidDataValue,
    /// The field contains unsupported currency
    InvalidDataCurrency,
    /// Unknown validation error code (for forward compatibility)
    #[serde(other)]
    Unknown,
}

/// Strike rs error
///
/// This enum represents all possible errors that can occur when using the Strike SDK.
#[derive(Debug, Error)]
pub enum Error {
    /// Not Found
    #[error("Not found")]
    NotFound,
    /// Invalid Url
    #[error("Invalid Url")]
    InvalidUrl(#[from] url::ParseError),
    /// From reqwest error
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    /// From serde error
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    /// Strike API error
    #[error("Strike API error: {0}")]
    ApiError(#[from] StrikeApiError),
}

impl std::fmt::Display for StrikeApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.data.status,
            format!("{:?}", self.data.code),
            self.data.message
        )?;

        if !self.data.validation_errors.is_empty() {
            write!(f, " (Validation errors: ")?;
            for (field, errors) in &self.data.validation_errors {
                for error in errors {
                    write!(f, "{}: {} ", field, error.message)?;
                }
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl std::error::Error for StrikeApiError {}

impl StrikeApiError {
    /// Get the HTTP status code
    pub fn status(&self) -> u16 {
        self.data.status
    }

    /// Get the error code
    pub fn code(&self) -> &StrikeErrorCode {
        &self.data.code
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.data.message
    }

    /// Get additional error values
    pub fn values(&self) -> &HashMap<String, serde_json::Value> {
        &self.data.values
    }

    /// Get validation errors
    pub fn validation_errors(&self) -> &HashMap<String, Vec<ValidationError>> {
        &self.data.validation_errors
    }

    /// Check if this is a specific error code
    pub fn is_error_code(&self, code: &StrikeErrorCode) -> bool {
        &self.data.code == code
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        self.data.code == StrikeErrorCode::InvalidData
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limit_error(&self) -> bool {
        matches!(
            self.data.code,
            StrikeErrorCode::RateLimitExceeded | StrikeErrorCode::TooManyAttempts
        )
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.data.status >= 500
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.data.status >= 400 && self.data.status < 500
    }

    /// Get a specific value from the error values
    pub fn get_value<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.data
            .values
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get the trace ID if available
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }
}

impl StrikeErrorCode {
    /// Get the typical HTTP status code for this error
    pub fn typical_status(&self) -> u16 {
        match self {
            StrikeErrorCode::NotFound => 404,
            StrikeErrorCode::InternalServerError => 500,
            StrikeErrorCode::BadGateway => 502,
            StrikeErrorCode::MaintenanceMode => 503,
            StrikeErrorCode::ServiceUnavailable => 503,
            StrikeErrorCode::GatewayTimeout => 504,
            StrikeErrorCode::RateLimitExceeded => 429,
            StrikeErrorCode::TooManyAttempts => 429,
            StrikeErrorCode::ProcessingConflict => 409,
            StrikeErrorCode::Unauthorized => 401,
            StrikeErrorCode::Forbidden => 403,
            StrikeErrorCode::InvalidData => 400,
            StrikeErrorCode::InvalidDataQuery => 400,
            StrikeErrorCode::UnprocessableEntity => 422,
            StrikeErrorCode::AccountNotReady => 425,
            StrikeErrorCode::DuplicateInvoice => 409,
            _ => 422, // Most other errors are 422
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StrikeErrorCode::InternalServerError
                | StrikeErrorCode::BadGateway
                | StrikeErrorCode::ServiceUnavailable
                | StrikeErrorCode::GatewayTimeout
                | StrikeErrorCode::RateLimitExceeded
                | StrikeErrorCode::TooManyAttempts
                | StrikeErrorCode::ProcessingConflict
                | StrikeErrorCode::AccountNotReady
                | StrikeErrorCode::LnUnavailable
                | StrikeErrorCode::ExchangeRateNotAvailable
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_deserialization() {
        let json = r#"{
            "traceId": "7cd128d3-e8a9-4685-b852-c8ce0f4dd5e3",
            "data": {
                "status": 422,
                "code": "USER_CURRENCY_UNAVAILABLE",
                "message": "User does not support the given currency.",
                "values": {
                    "currency": "USD"
                }
            }
        }"#;

        let error: StrikeApiError = serde_json::from_str(json).unwrap();
        assert_eq!(error.status(), 422);
        assert_eq!(error.code(), &StrikeErrorCode::UserCurrencyUnavailable);
        assert_eq!(
            error.get_value::<String>("currency"),
            Some("USD".to_string())
        );
    }

    #[test]
    fn test_validation_error_deserialization() {
        let json = r#"{
            "traceId": "7cd128d3-e8a9-4685-b852-c8ce0f4dd5e3",
            "data": {
                "status": 400,
                "validationErrors": {
                    "correlationId": [
                        {
                            "code": "INVALID_DATA_MAXLENGTH",
                            "message": "The field correlationId must be a value with a maximum length of '40'",
                            "values": {
                                "field": "correlationId",
                                "maxLength": 40
                            }
                        }
                    ]
                },
                "code": "INVALID_DATA",
                "message": "One or more validation errors occurred."
            }
        }"#;

        let error: StrikeApiError = serde_json::from_str(json).unwrap();
        assert!(error.is_validation_error());
        assert!(!error.validation_errors().is_empty());

        let correlation_errors = error.validation_errors().get("correlationId").unwrap();
        assert_eq!(correlation_errors.len(), 1);
        assert_eq!(
            correlation_errors[0].code,
            ValidationErrorCode::InvalidDataMaxlength
        );
    }
}
