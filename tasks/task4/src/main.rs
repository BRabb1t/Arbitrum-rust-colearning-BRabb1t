use alloy::network::EthereumWallet;
use alloy::primitives::utils::{format_units, parse_units};
use alloy::primitives::{Address, U256, U64};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{BlockId, BlockTransactionsKind, TransactionRequest};
use alloy::signers::local::PrivateKeySigner;
use eyre::{eyre, ensure, Result};
use std::env;
use std::str::FromStr;

const ARB_SEPOLIA_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";
const ARB_SEPOLIA_CHAIN_ID: u64 = 421614;
const AMOUNT_DEFAULT: &str = "0.001";
const TRANSFER_GAS_LIMIT: u64 = 21_000;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // 读取 RPC 与转账参数（从 .env / 环境变量中获取，避免硬编码私钥）
    let rpc_url = env::var("ARB_RPC_URL").unwrap_or_else(|_| ARB_SEPOLIA_RPC.to_string());
    let private_key = env::var("SENDER_PRIVATE_KEY")
        .map_err(|_| eyre!("missing env var: SENDER_PRIVATE_KEY"))?;
    let to_address = env::var("TO_ADDRESS").map_err(|_| eyre!("missing env var: TO_ADDRESS"))?;
    let amount_eth = env::var("AMOUNT_ETH").unwrap_or_else(|_| AMOUNT_DEFAULT.to_string());

    // 解析私钥与地址
    let signer: PrivateKeySigner = private_key.parse()?;
    let from_address = signer.address();
    let to_address = Address::from_str(&to_address)?;

    // 基础地址校验，避免零地址与自转
    ensure!(to_address != Address::ZERO, "TO_ADDRESS cannot be zero address");
    ensure!(
        to_address != from_address,
        "TO_ADDRESS must be different from sender"
    );

    // 使用钱包填充器构建 Provider，负责签名与补全链上字段
    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url.parse()?);

    // 校验链 ID，确保连接的是 Arbitrum Sepolia
    let chain_id = provider.get_chain_id().await?;
    ensure!(
        U64::from(ARB_SEPOLIA_CHAIN_ID) == chain_id,
        "RPC chain_id is not Arbitrum Sepolia (421614)"
    );

    // 转账金额（ETH -> wei）
    let value: U256 = parse_units(&amount_eth, "eth")?.into();
    // 动态获取 gas 价格并基于 base fee 计算 EIP-1559 费用
    let gas_price = provider.get_gas_price().await?;
    let latest_block = provider
        .get_block(BlockId::latest(), BlockTransactionsKind::Hashes)
        .await?
        .ok_or_else(|| eyre!("latest block not found"))?;
    let base_fee = latest_block
        .header
        .base_fee_per_gas
        .map(u128::from)
        .unwrap_or(gas_price);
    let max_priority_fee_per_gas = 1_000_000u128;
    let max_fee_per_gas = base_fee.saturating_add(max_priority_fee_per_gas);
    let gas_fee = U256::from(max_fee_per_gas) * U256::from(TRANSFER_GAS_LIMIT);

    // 组装交易并设置 gas 相关字段
    let mut tx = TransactionRequest::default()
        .to(to_address)
        .value(value)
        .gas_limit(TRANSFER_GAS_LIMIT);
    tx.max_fee_per_gas = Some(max_fee_per_gas);
    tx.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);

    let base_fee_gwei = format_units(U256::from(base_fee), "gwei")?;
    let max_fee_gwei = format_units(U256::from(max_fee_per_gas), "gwei")?;
    let gas_fee_eth = format_units(gas_fee, "eth")?;

    println!("RPC: {}", rpc_url);
    println!("From: {from_address:?}");
    println!("To: {to_address:?}");
    println!("Amount: {} ETH", amount_eth);
    println!(
        "Base fee: {} wei ({} gwei)",
        base_fee, base_fee_gwei
    );
    println!(
        "Max fee per gas: {} wei ({} gwei)",
        max_fee_per_gas, max_fee_gwei
    );
    println!("Gas limit: {}", TRANSFER_GAS_LIMIT);
    println!("Estimated fee: {} eth", gas_fee_eth);

    // 签名并发送交易，输出交易哈希
    let pending = provider.send_transaction(tx).await?;
    let tx_hash = *pending.tx_hash();
    println!("Tx hash: {tx_hash:?}");

    // 等待交易上链并打印回执信息
    let receipt = pending.get_receipt().await?;
    println!("Tx status: {}", receipt.status());
    println!("Tx in block: {:?}", receipt.block_number);

    Ok(())
}
