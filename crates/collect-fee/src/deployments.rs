use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignDeploymentsByFactory {
    pub factory_address: String,
    pub campaigns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentsByChain {
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
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string("src/deployments.json")?;
        let deployments: Deployments = serde_json::from_str(&content)?;
        Ok(deployments)
    }
}
