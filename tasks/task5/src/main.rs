use alloy::contract::Interface;
use alloy::dyn_abi::DynSolValue;
use alloy::json_abi::JsonAbi;
use alloy::primitives::{Address, Bytes};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{TransactionInput, TransactionRequest};
use eyre::{eyre, Result};
use std::env;
use std::str::FromStr;

const ARB_SEPOLIA_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";
const ARBSYS_ADDRESS: &str = "0x0000000000000000000000000000000000000064";
// --- 方案 A：arbOSVersion ---
const ARBSYS_ABI_OSVERSION: &str = r#"[{"inputs":[],"name":"arbOSVersion","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"}]"#;
// --- 方案 B：arbChainID ---
// const ARBSYS_ABI_CHAINID: &str = r#"[{"inputs":[],"name":"arbChainID","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"}]"#;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let rpc_url = env::var("ARB_RPC_URL").unwrap_or_else(|_| ARB_SEPOLIA_RPC.to_string());
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    // 加载合约 ABI 并连接 Arbitrum 预编译合约（ArbSys）
    // --- 方案 A：arbOSVersion ---
    let abi: JsonAbi = serde_json::from_str(ARBSYS_ABI_OSVERSION)?;
    // --- 方案 B：arbChainID ---
    // let abi: JsonAbi = serde_json::from_str(ARBSYS_ABI_CHAINID)?;
    let interface = Interface::new(abi);
    let contract_address = Address::from_str(ARBSYS_ADDRESS)?;

    // 编码只读函数调用数据
    // --- 方案 A：arbOSVersion ---
    let calldata = interface.encode_input("arbOSVersion", &[])?;
    // --- 方案 B：arbChainID ---
    // let calldata = interface.encode_input("arbChainID", &[])?;
    let tx = TransactionRequest::default()
        .to(contract_address)
        .input(TransactionInput::new(Bytes::from(calldata)));

    // eth_call 读取链上状态
    let raw = provider.call(&tx).await?;
    // --- 方案 A：arbOSVersion ---
    let decoded = interface.decode_output("arbOSVersion", &raw, true)?;
    // --- 方案 B：arbChainID ---
    // let decoded = interface.decode_output("arbChainID", &raw, true)?;
    let version = match decoded.as_slice() {
        [DynSolValue::Uint(value, _)] => *value,
        _ => return Err(eyre!("unexpected output for result")),
    };

    println!("RPC: {}", rpc_url);
    println!("Contract: {}", ARBSYS_ADDRESS);
    // --- 方案 A 输出（arbOSVersion）---
    println!("arbOSVersion: {}", version);
    // --- 方案 B 输出（arbChainID）---
    // println!("arbChainID: {}", version);

    Ok(())
}
