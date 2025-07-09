use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use strike_rs::{Currency, CurrencyExchangeQuoteRequest, ExchangeAmount, FeePolicy, Strike};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    let api_key = env::var("STRIKE_API_KEY").expect("STRIKE_API_KEY must be set");
    let client = Strike::new(&api_key, None)?;

    // Example 1: Exchange $5.00 USD for BTC
    println!("Creating currency exchange quote: $5.00 USD -> BTC");

    let exchange_request = CurrencyExchangeQuoteRequest {
        sell: Currency::USD,
        buy: Currency::BTC,
        amount: ExchangeAmount {
            amount: "5.00".to_string(),
            currency: Currency::USD,
            fee_policy: None,
        },
    };

    let quote = client
        .create_currency_exchange_quote(exchange_request)
        .await?;

    println!("Exchange quote created:");
    println!("  Quote ID: {}", quote.id);
    println!("  Sell: {} {}", quote.source.amount, quote.source.currency);
    println!("  Buy: {} {}", quote.target.amount, quote.target.currency);
    println!(
        "  Rate: {} {} per {}",
        quote.conversion_rate.amount,
        quote.conversion_rate.target_currency,
        quote.conversion_rate.source_currency
    );
    println!("  Valid until: {}", quote.valid_until);
    println!("  State: {:?}", quote.state);

    if let Some(fee) = &quote.fee {
        println!("  Fee: {} {}", fee.amount, fee.currency);
    }

    // Example 2: Create a quote with fee policy
    println!("\nCreating exchange quote with inclusive fee policy:");

    let exchange_request_with_fee = CurrencyExchangeQuoteRequest {
        sell: Currency::USD,
        buy: Currency::BTC,
        amount: ExchangeAmount {
            amount: "10.00".to_string(),
            currency: Currency::USD,
            fee_policy: Some(FeePolicy::Inclusive),
        },
    };

    let quote_with_fee = client
        .create_currency_exchange_quote(exchange_request_with_fee)
        .await?;

    println!("Exchange quote with fee policy created:");
    println!("  Quote ID: {}", quote_with_fee.id);
    println!(
        "  Sell: {} {}",
        quote_with_fee.source.amount, quote_with_fee.source.currency
    );
    println!(
        "  Buy: {} {}",
        quote_with_fee.target.amount, quote_with_fee.target.currency
    );

    // Example 3: Get quote details
    println!("\nRetrieving quote details:");
    let retrieved_quote = client.get_currency_exchange_quote(&quote.id).await?;
    println!("  Retrieved quote state: {:?}", retrieved_quote.state);

    // Example 4: Execute quote (commented out to avoid actual execution)
    // println!("\nExecuting currency exchange quote...");
    // client.execute_currency_exchange_quote(&quote.id).await?;
    // println!("Quote executed successfully!");

    // Example 5: Create a quote for buying a specific amount of BTC
    println!("\nCreating quote to buy 0.0001 BTC:");

    let btc_amount_request = CurrencyExchangeQuoteRequest {
        sell: Currency::USD,
        buy: Currency::BTC,
        amount: ExchangeAmount {
            amount: "0.0001".to_string(),
            currency: Currency::BTC,
            fee_policy: None,
        },
    };

    let btc_quote = client
        .create_currency_exchange_quote(btc_amount_request)
        .await?;

    println!("BTC amount quote created:");
    println!("  Quote ID: {}", btc_quote.id);
    println!(
        "  Sell: {} {}",
        btc_quote.source.amount, btc_quote.source.currency
    );
    println!(
        "  Buy: {} {}",
        btc_quote.target.amount, btc_quote.target.currency
    );

    Ok(())
}
