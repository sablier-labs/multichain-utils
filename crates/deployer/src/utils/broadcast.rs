use serde_json::Value;
use std::{fs, path::Path};

use super::chain_data;

pub struct Broadcast {
    file_path: String,
    chain_id: String,
    project: String,
    version: String,
}

impl Broadcast {
    // Creates a new `Broadcast` instance, initializing all fields
    pub fn new(
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
        } else if script_name.contains("MerkleFactory") || script_name.contains("MerkleFactories") {
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

        let file_path = if is_broadcast_deployment {
            format!("broadcast/{}/{}/run-latest.json", script_name, chain_id)
        } else {
            format!("broadcast/{}/{}/dry-run/run-latest.json", script_name, chain_id)
        };

        Some(Broadcast { chain_id, project, version, file_path })
    }

    // Helper to format and add rows to the table
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
            chain_data::get_explorer_url_with_address(&self.chain_id, contract_addr),
            &self.version,
            &self.project,
            &self.version
        );
        deployment_table.push_str(&row);
    }

    // Copy the broadcast file to the specified destination path
    pub fn copy_broadcast_file(
        &self,
        chain: &str,
    ) {
        let dest_path = format!("../deployments/{}/v{}/broadcasts/{}.json", self.project, self.version, chain);

        // Ensure the parent directory exists
        if let Some(parent) = Path::new(&dest_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create directories");
            }
        }

        // Copy the file
        fs::copy(&self.file_path, &dest_path).expect("Failed to copy and rename run-latest.json to deployments");
    }

    // Generate the deployment table for this broadcast
    pub fn generate_deployment_table(&self) -> String {
        // Read and parse the JSON content
        let json_content = fs::read_to_string(&self.file_path)
            .unwrap_or_else(|_| panic!("Failed to read the broadcast file: {}", &self.file_path));
        let json_value: Value = serde_json::from_str(&json_content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON in file: {}", &self.file_path));

        // Prepare the table headers
        let mut deployment_table = format!(
            "## {}\n\n| Contract | Address | Deployment |\n| :------- | :------ | :----------|\n",
            chain_data::get_name(&self.chain_id)
        );

        // The format of the JSON can be viewed here: https://github.com/sablier-labs/deployments/blob/main/
        // Extract returned contracts from the JSON file and add to the table
        if let Some(returned_obj) = json_value.get("returns").and_then(|v| v.as_object()) {
            returned_obj.values().for_each(|value| {
                if let (Some(internal_type), Some(contract_addr)) =
                    (value.get("internal_type").and_then(|v| v.as_str()), value.get("value").and_then(|v| v.as_str()))
                {
                    if let Some(contract_name) = internal_type.split_whitespace().last() {
                        self.add_to_table(&mut deployment_table, contract_addr, contract_name);
                    }
                }
            });
        }
        // Extract libraries and contracts from the JSON file and add to the table
        if let Some(libraries) = json_value.get("libraries").and_then(|v| v.as_array()) {
            libraries.iter().filter_map(|lib| lib.as_str()).for_each(|library_str| {
                let parts: Vec<&str> = library_str.split(':').collect();
                if parts.len() >= 3 {
                    self.add_to_table(&mut deployment_table, parts[2], parts[1]);
                }
            });
        }

        deployment_table.push('\n');
        deployment_table
    }
}
