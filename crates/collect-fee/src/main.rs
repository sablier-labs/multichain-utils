use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::{env, fs, time::SystemTime};
use tokio;

mod contracts;
mod deployments;
use contracts::{collect_fees_from_campaign, collect_fees_from_flow, collect_fees_from_lockup};
use deployments::Deployments;

#[derive(Debug, Serialize, Deserialize)]
struct Broadcast {
    tx_hash: String,
    timestamp: String,
    chain_id: String,
    chain_name: String,
    protocol: String,
    event_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Broadcasts {
    broadcasts: Vec<Broadcast>,
}

impl Broadcasts {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if !fs::metadata("broadcasts.json").is_ok() {
            let initial = Broadcasts { broadcasts: Vec::new() };
            initial.save()?;
        }
        let content = fs::read_to_string("broadcasts.json")?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("broadcasts.json", content)?;
        Ok(())
    }

    fn add_broadcast(
        &mut self,
        tx_hash: String,
        chain_id: String,
        chain_name: &str,
        protocol: String,
        event_data: serde_json::Value,
    ) {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let broadcast = Broadcast {
            tx_hash,
            timestamp: chrono::DateTime::from_timestamp(timestamp as i64, 0).unwrap().to_utc().to_rfc3339(),
            chain_id,
            chain_name: chain_name.to_string(),
            protocol,
            event_data,
        };
        self.broadcasts.push(broadcast);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load private key from .env
    let private_key = env::var("PRIVATE_KEY")?;
    let signer: PrivateKeySigner = private_key.parse().expect("should parse private key");
    let wallet = EthereumWallet::from(signer);

    // Load deployments
    let deployments = Deployments::load()?;

    let mut broadcasts = Broadcasts::load()?;

    // Process each chain
    for (chain_name, deployed_contracts) in deployments.chains {
        let rpc_url = env::var(format!("{}_RPC_URL", chain_name.to_uppercase()))?;
        let provider = ProviderBuilder::new().wallet(&wallet).on_http(rpc_url.parse()?);

        // Collect fees from lockup contracts
        for lockup_address in &deployed_contracts.lockup {
            let lockup: Address = lockup_address.parse()?;
            let ether_balance = provider.get_balance(lockup).await?;

            if !ether_balance.is_zero() {
                println!("Collecting fee from Lockup contract: {} on {}", lockup_address, chain_name);

                let (tx_hash, chain_id, event_data) = collect_fees_from_lockup(&rpc_url, &wallet, lockup).await?;
                broadcasts.add_broadcast(tx_hash, chain_id, &chain_name, "lockup".to_string(), event_data);
            }
        }

        // Collect fees from flow contracts
        for flow_address in &deployed_contracts.flow {
            let flow: Address = flow_address.parse()?;
            let ether_balance = provider.get_balance(flow).await?;

            if !ether_balance.is_zero() {
                println!("Collecting fee from Flow contract: {} on {}", flow_address, chain_name);

                let (tx_hash, chain_id, event_data) = collect_fees_from_flow(&rpc_url, &wallet, flow).await?;
                broadcasts.add_broadcast(tx_hash, chain_id, &chain_name, "flow".to_string(), event_data);
            }
        }

        // Collect fees from airdrop campaigns
        for (_, campaigns) in &deployed_contracts.merkle_contracts {
            let factory: Address = campaigns.factory_address.parse()?;
            for campaign_address in &campaigns.campaigns {
                let campaign = campaign_address.parse()?;
                let ether_balance = provider.get_balance(campaign).await?;

                if !ether_balance.is_zero() {
                    println!("Collecting fee from Campaign contract: {} on {}", campaign_address, chain_name);

                    let (tx_hash, chain_id, event_data) =
                        collect_fees_from_campaign(&rpc_url, &wallet, factory, campaign).await?;
                    broadcasts.add_broadcast(tx_hash, chain_id, &chain_name, "campaign".to_string(), event_data);
                }
            }
        }
    }

    // Save broadcasts
    broadcasts.save()?;

    Ok(())
}
