use alloy::primitives::utils::format_units;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use eyre::Result;
use std::str::FromStr;

pub async fn query_balance(address: &str) -> Result<String> {
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    let address = Address::from_str(address)?;
    let balance_wei = provider.get_balance(address).await?;
    let balance_eth = format_units(balance_wei, "eth")?;

    Ok(balance_eth)
}
