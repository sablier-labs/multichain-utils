use crate::{
    constants, rpc_methods,
    states::{BroadcastState, BroadcastsState, DeploymentsByChain},
};
use alloy::{
    primitives::{Address, U256},
    providers::Provider,
    rpc::types::{ReceiptEnvelope, TransactionReceipt},
    sol_types::SolEvent,
    transports::TransportErrorKind,
};

//////////////////////////////////////////////////////////////////////////
//                         SOLIDITY-INTERFACES
//////////////////////////////////////////////////////////////////////////

alloy::sol! {
    #[sol(rpc)]
    contract ISablierFactoryMerkle {
        #[derive(Debug)]
        function collectFees(address campaign) external;

        #[derive(Debug)]
        event CollectFees(address indexed admin, address indexed merkleBase, uint256 feeAmount);
    }
}

alloy::sol! {
    #[sol(rpc)]
    contract ISablierLockupAndFlow {
        #[derive(Debug)]
        function collectFees() external;

        #[derive(Debug)]
        event CollectFees(address indexed admin, uint256 indexed feeAmount);
    }
}

//////////////////////////////////////////////////////////////////////////
//                          COLLECT-FEE-ON-CHAIN
//////////////////////////////////////////////////////////////////////////

pub async fn collect_fees_on_chain<T: Provider>(
    provider: &T,
    deployments_by_chain: &DeploymentsByChain,
    chain_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut broadcasts = BroadcastsState::read_or_create(constants::DATA_FILE_BROADCASTS.to_string())?;

    // Collect fees from lockup contracts
    for lockup_address in &deployments_by_chain.lockup {
        let lockup: Address = lockup_address.parse()?;
        match collect_fees_generic(provider, chain_name, "lockup", lockup, None, &mut broadcasts).await {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }
    }

    // Collect fees from flow contracts
    for flow_address in &deployments_by_chain.flow {
        let flow: Address = flow_address.parse()?;
        match collect_fees_generic(provider, chain_name, "flow", flow, None, &mut broadcasts).await {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }
    }

    // Collect fees from airdrop campaigns
    for campaigns in deployments_by_chain.merkle_contracts.values() {
        let factory: Address = campaigns.factory_address.parse()?;
        for campaign_address in &campaigns.campaigns {
            let campaign: Address = campaign_address.parse()?;
            match collect_fees_generic(provider, chain_name, "campaign", campaign, Some(factory), &mut broadcasts).await
            {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            }
        }
    }

    Ok(())
}

//////////////////////////////////////////////////////////////////////////
//                              COLLECT-FEES
//////////////////////////////////////////////////////////////////////////

async fn collect_fees_generic<T: Provider>(
    provider: &T,
    chain_name: &str,
    contract_name: &str,
    contract_address: Address,
    factory_address: Option<Address>,
    broadcasts: &mut BroadcastsState,
) -> Result<(), Box<dyn std::error::Error>> {
    let fee_amount = match rpc_methods::fetch_balance_with_retry(provider, contract_address).await {
        Ok(amount) => amount,
        Err(e) => return Err(e),
    };

    // Check: if the fee amount is zero.
    if fee_amount.is_zero() {
        return Err(format!("No fees to collect from {}", contract_address).into());
    }

    let chain_id = match rpc_methods::get_chain_id_with_retry(provider).await {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let pending_tx = if let Some(factory) = factory_address {
        let contract = ISablierFactoryMerkle::new(factory, provider);

        contract.collectFees(contract_address).send().await.map_err(TransportErrorKind::custom)?
    } else {
        let contract = ISablierLockupAndFlow::new(contract_address, provider);

        contract.collectFees().send().await.map_err(TransportErrorKind::custom)?
    };

    println!("{} collecting from {} on {} named {}", fee_amount, contract_address, chain_name, contract_name);

    let receipt = pending_tx.get_receipt().await.map_err(TransportErrorKind::custom)?;

    let (admin, fee_amount) = handle_logs(&receipt)?;

    broadcasts.update_and_save(BroadcastState {
        admin,
        block_number: receipt.block_number.unwrap_or_default().to_string(),
        chain_id: chain_id.to_string(),
        chain_name: chain_name.into(),
        contract_address: contract_address.to_string(),
        contract_name: contract_name.to_string(),
        fee_collected: fee_amount,
        timestamp_utc: get_current_timestamp(),
        tx_hash: receipt.transaction_hash.to_string(),
    });

    Ok(())
}

//////////////////////////////////////////////////////////////////////////
//                               HELPERS
//////////////////////////////////////////////////////////////////////////

fn handle_logs(receipt: &TransactionReceipt) -> Result<(String, serde_json::Value), Box<dyn std::error::Error>> {
    // Match on the `inner` field to access the logs.
    let logs = match &receipt.inner {
        ReceiptEnvelope::Legacy(inner) |
        ReceiptEnvelope::Eip2930(inner) |
        ReceiptEnvelope::Eip1559(inner) |
        ReceiptEnvelope::Eip4844(inner) |
        ReceiptEnvelope::Eip7702(inner) => &inner.receipt.logs,
    };

    for log in logs {
        let inner_log = &log.inner;

        let (topics, data) = inner_log.data.clone().split();

        let mut admin_bytes = [0u8; 20];

        if topics[0] == ISablierFactoryMerkle::CollectFees::SIGNATURE_HASH {
            admin_bytes.copy_from_slice(&topics[1].as_slice()[12..]);
            let admin = Address::from_slice(&admin_bytes);

            // Decode the fee amount from the data field
            let fee_amount = U256::from_be_slice(&data[0..32]);

            // Return the admin address and fee amount.
            return Ok((admin.to_string(), serde_json::json!(fee_amount.to_string())));
        }

        if topics[0] == ISablierLockupAndFlow::CollectFees::SIGNATURE_HASH {
            admin_bytes.copy_from_slice(&topics[1].as_slice()[12..]);
            let admin = Address::from_slice(&admin_bytes);

            // Decode the fee amount from the data field
            let fee_amount = U256::from_be_slice(&topics[2].as_slice()[0..32]);

            // Return the admin address and fee amount.
            return Ok((admin.to_string(), serde_json::json!(fee_amount.to_string())));
        }
    }

    Err("No CollectFees found".into())
}

fn get_current_timestamp() -> String {
    chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
