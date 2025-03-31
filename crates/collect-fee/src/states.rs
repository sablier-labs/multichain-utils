use alloy::{network::EthereumWallet, primitives::Address, signers::local::PrivateKeySigner};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::{BufWriter, Write},
    path::Path,
};

//////////////////////////////////////////////////////////////////////////
//                             BROADCASTS-FILE
//////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastState {
    pub admin: String,
    pub block_number: String,
    pub chain_id: String,
    pub chain_name: String,
    pub contract_address: String,
    pub contract_name: String,
    pub fee_collected: Value,
    pub timestamp_utc: String,
    pub tx_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastsState {
    file_name: String,
    broadcasts: Vec<BroadcastState>,
}

impl BroadcastsState {
    pub fn read_or_create(file_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent_dir) = Path::new(&file_name).parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        if Path::new(&file_name).exists() {
            let content = std::fs::read_to_string(&file_name)?;
            let content: Vec<BroadcastState> = serde_json::from_str(&content)?;
            Ok(Self { file_name, broadcasts: content })
        } else {
            let mut empty_file = Self { file_name: file_name.clone(), broadcasts: vec![] };
            let _ = empty_file.save_to_file();
            Ok(empty_file)
        }
    }

    pub fn update_and_save(
        &mut self,
        new_broadcast: BroadcastState,
    ) {
        self.broadcasts.push(new_broadcast);
        let _ = self.save_to_file();
    }

    fn save_to_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string_pretty(&self.broadcasts)?;

        let tmp_path = format!("{}.tmp", &self.file_name);
        let mut writer = BufWriter::new(std::fs::File::create(&tmp_path)?);
        writer.write_all(serialized.as_bytes())?;
        writer.flush()?;
        std::fs::rename(tmp_path, &self.file_name)?; // atomic replace

        Ok(())
    }
}

//////////////////////////////////////////////////////////////////////////
//                              DEPLOYMENTS-FILE
//////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignDeploymentsByFactory {
    pub factory_address: String,
    pub campaigns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentsByChain {
    pub chain_id: u64,
    pub lockup: Vec<String>,
    pub flow: Vec<String>,
    #[serde(flatten)]
    pub merkle_contracts: HashMap<String, CampaignDeploymentsByFactory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Deployments {
    #[serde(flatten)]
    pub chains: HashMap<String, DeploymentsByChain>,
}

impl Deployments {
    pub fn load_from_file(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_name)?;
        let deployments: Self = serde_json::from_str(&content)?;

        Ok(deployments)
    }
}

//////////////////////////////////////////////////////////////////////////
//                               EOA-WALLET
//////////////////////////////////////////////////////////////////////////

pub struct EOAWallet {
    pub address: Address,
    pub signer: EthereumWallet,
}

impl EOAWallet {
    pub fn new(private_key: String) -> Self {
        // Check that the private key is valid.
        let pk_signer: PrivateKeySigner = private_key.parse().expect("invalid private key");

        EOAWallet { address: pk_signer.clone().address(), signer: EthereumWallet::from(pk_signer) }
    }
}
