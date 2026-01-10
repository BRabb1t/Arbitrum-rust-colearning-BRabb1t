use alloy::primitives::utils::format_units;
use alloy::primitives::U256;
use alloy::providers::{Provider, ProviderBuilder};
use eyre::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = env::var("ARB_RPC_URL")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    // 通过 RPC 动态获取当前 Gas 价格（单位：wei）
    let gas_price = provider.get_gas_price().await?;
    // 将 u128 的 gas_price 转为 U256 便于后续计算
    let gas_price_u256 = U256::from(gas_price);
    // 基础转账 Gas 限额（行业通用值）
    let gas_limit = U256::from(21_000u64);
    // 预估 Gas 费 = Gas 价格 × Gas 限额
    let gas_fee = gas_price_u256 * gas_limit;

    let gas_price_gwei = format_units(gas_price_u256, "gwei")?;
    let gas_fee_gwei = format_units(gas_fee, "gwei")?;
    let gas_fee_eth = format_units(gas_fee, "eth")?;

    println!("RPC: {}", rpc_url);
    println!("Gas price: {} wei ({} gwei)", gas_price, gas_price_gwei);
    println!("Gas limit: {}", gas_limit);
    println!(
        "Estimated fee (gas_price * gas_limit): {} wei ({} gwei, {} eth)",
        gas_fee, gas_fee_gwei, gas_fee_eth
    );

    Ok(())
}
