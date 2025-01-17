use serde_json::Value;
use std::{env, fs, io::Write, path::Path, process::Command};
use toml::Value as TomlValue;

struct Broadcast {
    file_path: String,
    chain_id: String,
    project: String,
    version: String,
}

impl Broadcast {
    /// Creates a new `Broadcast` instance, initializing all fields
    fn new(
        output_str: &str,
        script_name: &str,
        is_broadcast_deployment: bool,
    ) -> Option<Self> {
        // Extract the chain ID
        let chain_id = output_str
            .lines()
            .find(|line| line.trim().starts_with("Chain "))
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("")
            .to_string();

        // Determine the project name
        let project = if script_name.contains("Protocol") || script_name.contains("Lockup") {
            "lockup".to_string()
        } else if script_name.contains("Flow") {
            "flow".to_string()
        } else if script_name.contains("MerkleFactory") {
            "airdrops".to_string()
        } else {
            // Return None if the script name doesn't match any of the above
            return None;
        };

        // Read the version from package.json
        let version = serde_json::from_str::<Value>(&fs::read_to_string("package.json").unwrap()).unwrap()["version"]
            .as_str()
            .unwrap()
            .to_string();

        let file_path;

        // Determine the file path
        if is_broadcast_deployment {
            file_path = format!("broadcast/{}/{}/run-latest.json", script_name, chain_id)
        } else {
            file_path = format!("broadcast/{}/{}/dry-run/run-latest.json", script_name, chain_id)
        }

        Some(Broadcast { chain_id, project, version, file_path })
    }

    /// Copy the broadcast file to the specified destination path
    fn copy_broadcast_file(
        &self,
        chain: &str,
    ) {
        let dest_path = format!("../v2-deployments/{}/v{}/broadcasts/{}.json", self.project, self.version, chain);

        // Ensure the parent directory exists
        if let Some(parent) = Path::new(&dest_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create directories");
            }
        }

        // Copy the file
        fs::copy(&self.file_path, &dest_path).expect("Failed to copy and rename run-latest.json to v2-deployments");
    }

    /// Generate the deployment table for this broadcast
    fn generate_deployment_table(&self) -> String {
        // Read and parse the JSON content
        let json_content = fs::read_to_string(&self.file_path)
            .unwrap_or_else(|_| panic!("Failed to read the broadcast file: {}", &self.file_path));
        let json_value: Value = serde_json::from_str(&json_content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON in file: {}", &self.file_path));

        // Prepare the table headers
        let mut deployment_table = format!(
            "## {}\n\n| Contract | Address | Deployment |\n| :------- | :------ | :----------|\n",
            chain_name(&self.chain_id)
        );

        // Extract libraries and contracts, and add to the table
        json_value.get("libraries").and_then(|v| v.as_array()).map(|libraries| {
            libraries.iter().filter_map(|lib| lib.as_str()).for_each(|library_str| {
                let parts: Vec<&str> = library_str.split(':').collect();
                if parts.len() >= 3 {
                    self.add_to_table(&mut deployment_table, parts[2], parts[1]);
                }
            });
        });

        json_value.get("returns").and_then(|v| v.as_object()).map(|returned_obj| {
            returned_obj.values().for_each(|value| {
                if let (Some(internal_type), Some(contract_addr)) =
                    (value.get("internal_type").and_then(|v| v.as_str()), value.get("value").and_then(|v| v.as_str()))
                {
                    if let Some(contract_name) = internal_type.split_whitespace().last() {
                        self.add_to_table(&mut deployment_table, contract_addr, contract_name);
                    }
                }
            });
        });

        deployment_table.push('\n');
        deployment_table
    }

    /// Helper to format and add rows to the table
    fn add_to_table(
        &self,
        deployment_table: &mut String,
        contract_addr: &str,
        contract_name: &str,
    ) {
        let row = format!(
            "| {} | [{}]({}) | [v{}](https://github.com/sablier-labs/deployments/blob/main/{}/v{}) |\n",
            contract_name,
            contract_addr,
            explorer_url(&self.chain_id, contract_addr),
            &self.version,
            &self.project,
            &self.version
        );
        deployment_table.push_str(&row);
    }
}

pub fn chain_name(chain_id: &str) -> &str {
    match chain_id {
        // Mainnets
        "1" => "Ethereum Mainnet",
        "2741" => "Abstract",
        "42161" => "Arbitrum One",
        "43114" => "Avalanche",
        "8453" => "Base",
        "81457" => "Blast",
        "56" => "BNB Smart Chain",
        "88888" => "Chiliz",
        "1116" => "Core Dao",
        "100" => "Gnosis",
        "4689" => "IoTex",
        "1890" => "Lightlink",
        "59144" => "Linea",
        "333000333" => "Meld",
        "34443" => "Mode",
        "2818" => "Morph",
        "10" => "Optimism",
        "137" => "Polygon",
        "534352" => "Scroll",
        "5330" => "Superseed",
        "167000" => "Taiko Mainnet",
        "5845" => "Tangle",
        "324" => "zkSync Era",
        // Testnets
        "11155111" => "Sepolia",
        "421614" => "Arbitrum Sepolia",
        "84532" => "Base Sepolia",
        "80084" => "Berachain Bartio",
        "168587773" => "Blast Sepolia",
        "59141" => "Linea Sepolia",
        "919" => "Mode Sepolia",
        "2810" => "Morph Holesky",
        "11155420" => "Optimism Sepolia",
        "974399131" => "SKALE Testnet",
        "53302" => "Superseed Sepolia",
        "167009" => "Taiko Hekla",
        "300" => "zkSync Sepolia",
        // Return chain_id if no match found
        _ => chain_id,
    }
}

pub fn explorer_url(
    chain_id: &str,
    contract_addr: &str,
) -> String {
    let base_url = match chain_id {
        // Mainnets
        "1" => "https://etherscan.io/address/",
        "2741" => "https://wwstage.abscan.org/address/",
        "42161" => "https://arbiscan.io/address/",
        "43114" => "https://snowtrace.io/address/",
        "8453" => "https://basescan.org/address/",
        "81457" => "https://blastscan.io/address/",
        "56" => "https://bscscan.com/address/",
        "88888" => "https://chiliscan.com/address/",
        "1116" => "https://scan.coredao.org/address/",
        "100" => "https://gnosisscan.io/address/",
        "4689" => "https://iotexscan.io/address/",
        "1890" => "https://phoenix.lightlink.io/address/",
        "59144" => "https://lineascan.build/address/",
        "333000333" => "https://meldscan.io/address/",
        "34443" => "https://explorer.mode.network/address/",
        "2818" => "https://explorer.morphl2.io/address/",
        "10" => "https://optimistic.etherscan.io/address/",
        "137" => "https://polygonscan.com/address/",
        "534352" => "https://scrollscan.com/address/",
        "5330" => "https://explorer.superseed.xyz/address/",
        "167000" => "https://taikoscan.io/address/",
        "5845" => "https://explorer.tangle.tools/address/",
        "324" => "https://era.zksync.network/address/",
        // Testnets
        "11155111" => "https://sepolia.etherscan.io/address/",
        "421614" => "https://sepolia.arbiscan.io/address/",
        "84532" => "https://sepolia.basescan.org/address/",
        "80084" => "https://sepolia.berachain.io/address/",
        "168587773" => "https://sepolia.blastscan.io/address/",
        "59141" => "https://sepolia.lineascan.build/address/",
        "919" => "https://sepolia.mode.network/address/",
        "2810" => "https://sepolia.morphl2.io/address/",
        "11155420" => "https://sepolia.optimistic.etherscan.io/address/",
        "974399131" => "https://sepolia.skale.io/address/",
        "53302" => "https://sepolia.superseed.xyz/address/",
        "167009" => "https://sepolia.taikoscan.io/address/",
        "300" => "https://sepolia.zksync.network/address/",
        // Return a placeholder if no match found
        _ => "<N/A>",
    };

    if base_url == "<N/A>" {
        base_url.to_string()
    } else {
        format!("{}{}", base_url, contract_addr)
    }
}
