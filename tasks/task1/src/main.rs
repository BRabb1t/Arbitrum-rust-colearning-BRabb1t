use alloy::providers::{Provider, ProviderBuilder};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    println!("Hello web3!");
    println!("Connecting to Arbitrum Sepolia...");
    let block_number = provider.get_block_number().await?;
    println!("Latest block number: {}", block_number);

    Ok(())
}
