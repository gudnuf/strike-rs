use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use strike_rs::{Currency, CurrencyExchangeQuoteRequest, ExchangeAmount, Strike};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    let api_key = env::var("STRIKE_API_KEY").expect("STRIKE_API_KEY must be set");
    let client = Strike::new(&api_key, None)?;

    // Create a channel to receive webhook notifications
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Create webhook router for currency exchange events
    let webhook_router = client
        .create_currency_exchange_webhook_router("/currency-exchange-webhook", tx)
        .await?;

    // Subscribe to currency exchange webhook events
    let webhook_url = "https://your-webhook-endpoint.com/currency-exchange-webhook".to_string();
    client
        .subscribe_to_currency_exchange_webhook(webhook_url)
        .await?;

    println!("Webhook subscription created for currency exchange events");

    // Create a currency exchange quote
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
    println!("Created currency exchange quote: {}", quote.id);

    // Start the webhook server (in a real application, you would bind this to a port)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Webhook server listening on http://127.0.0.1:3000");

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, webhook_router).await.unwrap();
    });

    // Listen for webhook events
    tokio::spawn(async move {
        while let Some(quote_id) = rx.recv().await {
            println!(
                "Received webhook notification for currency exchange quote: {}",
                quote_id
            );

            // In a real application, you would fetch the updated quote status here
            // let updated_quote = client.get_currency_exchange_quote(&quote_id).await?;
            // println!("Updated quote state: {:?}", updated_quote.state);
        }
    });

    // Execute the quote after a short delay (optional)
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Uncomment to actually execute the quote
    // println!("Executing currency exchange quote...");
    // client.execute_currency_exchange_quote(&quote.id).await?;
    // println!("Quote executed!");

    // Keep the server running
    println!("Server running... Press Ctrl+C to stop");
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    Ok(())
}
