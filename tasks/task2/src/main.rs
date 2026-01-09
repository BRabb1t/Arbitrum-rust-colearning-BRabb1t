#[path = "level2-balance-query/balance.rs"]
mod balance;

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    let eth_balance = balance::query_balance(&address).await?;

    println!("Address: {}", address);
    println!("Balance: {} ETH", eth_balance);

    Ok(())
}
