use serde_json::Value;
use std::{fs, path::Path};

pub struct Broadcast {
    file_path: String,
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

        Some(Broadcast { project, version, file_path })
    }

    // Copy the broadcast file to the specified destination path
    pub fn copy_broadcast_file(
        &self,
        chain: &str,
    ) {
        let dest_path = format!("../sdk/{}/v{}/broadcasts/{}.json", self.project, self.version, chain);

        // Ensure the parent directory exists
        if let Some(parent) = Path::new(&dest_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create directories");
            }
        }

        // Copy the file
        fs::copy(&self.file_path, &dest_path).expect("Failed to copy and rename run-latest.json to sdk");
    }
}
