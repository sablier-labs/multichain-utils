use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    sol,
    transports::TransportErrorKind,
};
use serde_json::Value;

sol! {
    #[sol(rpc)]
    contract ISablierFactoryMerkle {
        #[derive(Debug)]
        function collectFees(address campaign) external;
    }
}

sol! {
    #[sol(rpc)]
    contract ISablierLockupAndFlow {
        #[derive(Debug)]
        function collectFees() external;
    }
}

async fn collect_fees(
    rpc_url: &str,
    wallet: &EthereumWallet,
    contract_address: Address,
) -> Result<(String, String, Value), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url.parse()?);

    let chain_id = provider.get_chain_id().await?;

    let contract = ISablierLockupAndFlow::new(contract_address, &provider);

    let pending_tx = contract.collectFees().send().await.map_err(TransportErrorKind::custom)?;
    let receipt = pending_tx.get_receipt().await.map_err(TransportErrorKind::custom)?;

    // Match on the `inner` field to access the logs
    let logs = match &receipt.inner {
        alloy::rpc::types::ReceiptEnvelope::Legacy(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip2930(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip1559(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip4844(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip7702(inner) => &inner.receipt.logs,
    };

    // Iterate over the logs and read the topics
    for log in logs {
        // Access the inner log
        let inner_log = &log.inner;

        // Access the topics
        let topics = &inner_log.data.topics();

        // Decode the admin address from topics[1]
        let mut admin_bytes = [0u8; 20];
        admin_bytes.copy_from_slice(&topics[1].as_slice()[12..]); // Extract last 20 bytes
        let admin = Address::from_slice(&admin_bytes);

        // Decode the fee amount from the data field
        let fee_amount = U256::from_be_slice(&topics[2].as_slice()[0..32]); // First 32 bytes as uint256

        println!("Fee amount: {}", fee_amount);

        // Return the decoded values
        return Ok((
            receipt.transaction_hash.to_string(),
            chain_id.to_string(),
            serde_json::json!({
            "admin": format!("{:#x}", admin),
            "feeAmount": fee_amount.to::<u128>(), // Convert to integer
            "contract_address": contract_address.to_string(),
            "blockNumber": receipt.block_number.unwrap_or_default().to_string()
            }),
        ));
    }

    // If no matching log is found, return an error or a default result
    Err("No matching CollectFees event found in logs".into())
}

pub async fn collect_fees_from_flow(
    rpc_url: &str,
    wallet: &EthereumWallet,
    flow_address: Address,
) -> Result<(String, String, Value), Box<dyn std::error::Error>> {
    collect_fees(rpc_url, wallet, flow_address).await
}

pub async fn collect_fees_from_lockup(
    rpc_url: &str,
    wallet: &EthereumWallet,
    lockup_address: Address,
) -> Result<(String, String, Value), Box<dyn std::error::Error>> {
    collect_fees(rpc_url, wallet, lockup_address).await
}

pub async fn collect_fees_from_campaign(
    rpc_url: &str,
    wallet: &EthereumWallet,
    factory_address: Address,
    campaign_address: Address,
) -> Result<(String, String, Value), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url.parse()?);

    let chain_id = provider.get_chain_id().await?;

    let contract = ISablierFactoryMerkle::new(factory_address, &provider);

    let pending_tx = contract.collectFees(campaign_address).send().await.map_err(TransportErrorKind::custom)?;

    // print pending_tx
    let receipt = pending_tx.get_receipt().await.map_err(TransportErrorKind::custom)?;

    // Match on the `inner` field to access the logs
    let logs = match &receipt.inner {
        alloy::rpc::types::ReceiptEnvelope::Legacy(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip2930(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip1559(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip4844(inner) |
        alloy::rpc::types::ReceiptEnvelope::Eip7702(inner) => &inner.receipt.logs,
    };

    // Iterate over the logs and read the topics
    for log in logs {
        // Access the inner log
        let inner_log = &log.inner;

        // Access the topics
        let (topics, data) = inner_log.data.clone().split();

        // Decode the admin address from topics[1]
        let mut admin_bytes = [0u8; 20];
        admin_bytes.copy_from_slice(&topics[1].as_slice()[12..]); // Extract last 20 bytes
        let admin = Address::from_slice(&admin_bytes);

        // Decode the fee amount from the data field
        let fee_amount = U256::from_be_slice(&data[0..32]); // First 32 bytes as uint256

        println!("Fee amount: {}", fee_amount);

        // Return the decoded values
        return Ok((
            receipt.transaction_hash.to_string(),
            chain_id.to_string(),
            serde_json::json!({
            "admin": format!("{:#x}", admin),
            "feeAmount": fee_amount.to::<u128>(), // Convert to integer
            "contract_address": campaign_address.to_string(),
            "blockNumber": receipt.block_number.unwrap_or_default().to_string()
            }),
        ));
    }

    // If no matching log is found, return an error or a default result
    Err("No matching CollectFees event found in logs".into())
}
