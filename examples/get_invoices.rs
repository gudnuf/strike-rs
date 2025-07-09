use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use strike_rs::{Filter, InvoiceQueryParams, Strike};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv().ok();

    let api_key = env::var("STRIKE_API_KEY").expect("STRIKE_API_KEY must be set");
    let client = Strike::new(&api_key, None)?;

    // Example 1: Get all invoices without any filters
    println!("Getting all invoices...");
    let invoices = client.get_invoices(None).await?;
    println!("Found {} invoices", invoices.count);
    for invoice in &invoices.items {
        println!(
            "Invoice ID: {}, State: {:?}, Amount: {} {}",
            invoice.invoice_id, invoice.state, invoice.amount.amount, invoice.amount.currency
        );
    }

    // Example 2: Get invoices with pagination
    println!("\nGetting first 5 invoices...");
    let params = InvoiceQueryParams::new().top(5).skip(0);
    let invoices = client.get_invoices(Some(params)).await?;
    println!("Found {} invoices (showing first 5)", invoices.count);

    // Example 3: Get invoices with filtering
    println!("\nGetting paid invoices...");
    let params = InvoiceQueryParams::new()
        .filter(Filter::eq("state", "PAID"))
        .orderby("created desc".to_string())
        .top(10);
    let invoices = client.get_invoices(Some(params)).await?;
    println!("Found {} paid invoices", invoices.count);

    // Example 4: Get invoices with complex filter
    println!("\nGetting USD invoices...");
    let params = InvoiceQueryParams::new()
        .filter(Filter::eq("currency", "USD"))
        .orderby("created desc".to_string());
    let invoices = client.get_invoices(Some(params)).await?;
    println!("Found {} USD invoices", invoices.count);

    Ok(())
}
