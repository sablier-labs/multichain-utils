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
        } else if script_name.contains("Merkle") || script_name.contains("Factories") {
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

    fn push_to_deployments_file(
        &self,
        deployments_file: &mut String,
        chain: &str,
        name: &str,
        address: &str,
        block_number: &str,
    ) {
        let raw = format!("chains.{}.id, {{ [manifest.{}]: [\"{}\", {}]", chain, name, address, block_number);
        deployments_file.push_str(&raw);
    }

    pub fn get_deployments_file(
        &self,
        chain: &str,
    ) -> String {
        // Read and parse the JSON content
        let json_content = fs::read_to_string(&self.file_path)
            .unwrap_or_else(|_| panic!("Failed to read the broadcast file: {}", &self.file_path));
        let json_value: Value = serde_json::from_str(&json_content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON in file: {}", &self.file_path));

        // format!(
        //     "/**\n \
        //          * @description Mainnet deployments for {} v{}\n \
        //          *\n \
        //          */\n \
        //          export const mainnets: Sablier.Deployment[] = [\n {} \n];\n \
        //          export const testnets: Sablier.Deployment[] = [\n {} \n];",
        //     self.project, self.version
        // );

        let mut deployments_file = "get( \n".to_string();

        // The format of the JSON can be viewed here: https://github.com/sablier-labs/deployments/blob/main/
        // Extract returned contracts from the JSON file
        if let Some(returned_obj) = json_value.get("returns").and_then(|v| v.as_object()) {
            returned_obj.values().for_each(|value| {
                if let (Some(internal_type), Some(contract_addr)) =
                    (value.get("internal_type").and_then(|v| v.as_str()), value.get("value").and_then(|v| v.as_str()))
                {
                    if let Some(contract_name) = internal_type.split_whitespace().last() {
                        let snake_case_name = self.get_snake_case_name(contract_name);
                        let block_number = self.get_block_number(&json_value);
                        self.push_to_deployments_file(
                            &mut deployments_file,
                            chain,
                            &snake_case_name,
                            contract_addr,
                            &block_number,
                        );
                    }
                }
            });
        }
        // Extract libraries and contracts from the JSON file
        if let Some(libraries) = json_value.get("libraries").and_then(|v| v.as_array()) {
            libraries.iter().filter_map(|lib| lib.as_str()).for_each(|library_str| {
                let parts: Vec<&str> = library_str.split(':').collect();
                if parts.len() >= 3 {
                    let snake_case_name = self.get_snake_case_name(parts[1]);
                    let block_number = self.get_block_number(&json_value);
                    self.push_to_deployments_file(
                        &mut deployments_file,
                        chain,
                        &snake_case_name,
                        parts[2],
                        &block_number,
                    );
                }
            });
        }

        deployments_file.push_str("\n}),");

        // return the deployments file
        deployments_file
    }

    fn get_snake_case_name(
        &self,
        name: &str,
    ) -> String {
        let mut out = String::new();
        let chars: Vec<char> = name.chars().collect();

        for i in 0..chars.len() {
            let ch = chars[i];
            if ch.is_uppercase() {
                // insert underscore if not first char
                // and previous char is lowercase OR next char is lowercase
                if i > 0 && (chars[i - 1].is_lowercase() || (i + 1 < chars.len() && chars[i + 1].is_lowercase())) {
                    out.push('_');
                }
            }
            out.push(ch.to_ascii_uppercase());
        }

        out
    }

    fn get_block_number(
        &self,
        json_value: &Value,
    ) -> String {
        json_value["receipts"][0]["blockNumber"]
            .as_str()
            .and_then(|hex| u64::from_str_radix(hex.trim_start_matches("0x"), 16).ok())
            .map(|n| n.to_string())
            .unwrap_or_default()
    }
}
