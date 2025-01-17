use serde_json::Value;
use std::{env, fs, io::Write, path::Path, process::Command};
use toml::Value as TomlValue;

mod chain_map;

struct Broadcast {
    file_path: String,
    chain_id: String,
    project: String,
    version: String,
}

impl Broadcast {
    fn new() -> Self {
        Broadcast { file_path: String::new(), chain_id: String::new(), project: String::new(), version: String::new() }
    }
}

fn main() {
    // Process command-line arguments
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter().skip(1);

    // Variables to store flags and provided chains
    let mut broadcast_deployment = "".to_string();
    let mut cp_broadcasted_file = false;
    let mut gas_price = "".to_string();
    let mut log_broadcasts = false;
    let mut on_all_chains = false;
    let mut provided_chains = Vec::new();
    let mut script_name = "".to_string();
    let mut verify_deployment = false;

    // Initialize Broadcast struct
    let mut broadcast = Broadcast::new();

    // Parse all arguments
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--all" => on_all_chains = true,
            "--broadcast" => broadcast_deployment = "--broadcast".to_string(),
            "--cp-bf" => cp_broadcasted_file = true,
            "--gas-price" => {
                let value = iter.next().expect("gas price value").to_string();
                gas_price = format!(" --gas-price {}", value);
            }
            "--log" => log_broadcasts = true,
            "--script" => {
                script_name = iter.next().expect("script name").to_string();
            }
            "--verify" => verify_deployment = true,
            _ => {
                if !arg.starts_with("--") && !on_all_chains {
                    provided_chains.push(arg.to_string());
                } else {
                    println!("Unknown flag: {}", arg);
                }
            }
        }
    }

    // Use a default script if no script name is provided
    if script_name.is_empty() {
        script_name = "DeployProtocol.s.sol".to_string();
        println!("No script name provided, using default script: {}\n", script_name);
    }

    let chains = get_all_chains();

    if on_all_chains {
        provided_chains = chains;
    } else {
        // Filter out chains that are not configured in the TOML file
        provided_chains.retain(|chain| {
            if chains.contains(chain) {
                true // Keep the element in the vector
            } else {
                println!("Chain {} is not configured in the TOML file \n", chain);
                false // Remove the element from the vector
            }
        });
    }

    // Default to "sepolia" if no chains are specified and --all is not used
    if provided_chains.is_empty() && !on_all_chains {
        provided_chains.push("sepolia".to_string());
    }

    // Output the list of unique chains
    let chains_string = provided_chains.clone().join(", ");
    println!("\nDeploying to the chains: {}\n", chains_string);

    // Delete the deployment file if it exists
    let deployment_file = "deployments.md";
    if fs::metadata(deployment_file).is_ok() {
        fs::remove_file(deployment_file).expect("Cant delete deployment file");
    }

    for chain in provided_chains {
        let env_var = "FOUNDRY_PROFILE=optimized";
        let command = "forge";

        let mut command_args =
            vec!["script".to_string(), format!("script/{}", script_name), "--rpc-url".to_string(), chain.to_string()];

        if !broadcast_deployment.is_empty() {
            command_args.push(broadcast_deployment.to_string());
        }

        if !gas_price.is_empty() {
            command_args.push(gas_price.to_string());
        }

        // Push the verify flag and etherscan API key. We need to it separately because otherwise they would be treated
        // as a single argument.
        if verify_deployment {
            command_args.push("--verify".to_string());
            command_args.push("--etherscan-api-key".to_string());
            command_args.push(format!("${}_API_KEY", chain.to_uppercase()));
        }

        println!("Running the deployment command: {} {} {} \n", env_var, command, command_args.join(" "));

        // Set the environment variable
        let env_var_parts: Vec<&str> = env_var.split('=').collect();
        env::set_var(env_var_parts[0], env_var_parts[1]);

        // Create the CLI and capture the command output
        let output = Command::new(command).args(&command_args).output().expect("Failed to run command");

        // Process command output
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output.status.success() {
            println!("Command output: {}\n", output_str);
        } else {
            eprintln!("Command failed with error: {}\n", String::from_utf8_lossy(&output.stderr));
        }

        // Identify the chain_id from the output
        broadcast.chain_id = output_str
            .lines()
            .find(|line| line.trim().starts_with("Chain "))
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("")
            .to_string();

        // Identify the project name
        broadcast.project = if script_name.contains("Protocol") || script_name.contains("Lockup") {
            "lockup".to_string()
        } else if script_name.contains("Flow") {
            "flow".to_string()
        } else if script_name.contains("MerkleFactory") {
            "airdrops".to_string()
        } else {
            // skip this function if the script name doesn't match any of the above
            return;
        };

        // Read the version from package.json
        broadcast.version = serde_json::from_str::<Value>(&fs::read_to_string("package.json").unwrap()).unwrap()
            ["version"]
            .as_str()
            .unwrap()
            .to_string();

        // Read the broadcast file
        broadcast.file_path = read_broadcast_file(&broadcast.chain_id, !broadcast_deployment.is_empty(), &script_name);

        if cp_broadcasted_file {
            // Copy broadcast file
            let dest_path =
                format!("../v2-deployments/{}/v{}/broadcasts/{}.json", broadcast.project, &broadcast.version, chain);
            copy_broadcast_file(&broadcast.file_path, &dest_path);
        }

        if log_broadcasts {
            // Generate the deployment table
            let deployment_table = generate_deployment_table(&broadcast);

            // Append the deployment table to the file
            let mut file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(deployment_file)
                .expect("Failed to open deployment file");
            file.write_all(deployment_table.as_bytes()).expect("Failed to write to the deployment file");
        }
    }
}

// Create the row and enter it into the table
fn add_to_table(
    deployment_table: &mut String,
    broadcast: &Broadcast,
    contract_addr: &str,
    contract_name: &str,
) {
    let row = format!(
        "| {} | [{}]({}) | [v{}](https://github.com/sablier-labs/deployments/blob/main/{}/v{}) |",
        contract_name,
        contract_addr,
        chain_map::explorer_url(&broadcast.chain_id, contract_addr),
        &broadcast.version,
        &broadcast.project,
        &broadcast.version
    );
    deployment_table.push_str(&format!("{}\n", row.as_str()));
}

// Copy the broadcast file to the dest_path
fn copy_broadcast_file(
    src_path: &str,
    dest_path: &str,
) {
    // Create the parent directory if it doesn't exist
    if let Some(parent) = Path::new(&dest_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
    }

    // Copy and rename the file
    fs::copy(src_path, dest_path).expect("Failed to copy and rename run-latest.json to v2-deployments\n");
}

// Generate the deployment table
fn generate_deployment_table(broadcast: &Broadcast) -> String {
    // Read the broadcast JSON object.
    let json_content = fs::read_to_string(&broadcast.file_path).expect("Failed to read the broadcast file");
    let json_value: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON");

    // Prepare the table headers.
    let mut deployment_table = format!(
        "## {}\n\n| Contract | Address | Deployment |\n| :------- | :------ | :----------|\n",
        chain_map::chain_name(&broadcast.chain_id)
    );

    // Look for libraries and add to table if found.
    if let Some(libraries) = json_value.get("libraries").and_then(|v| v.as_array()) {
        for library in libraries {
            match library.as_str() {
                Some(library_str) => {
                    let library_name = library_str.split(':').collect::<Vec<&str>>()[1];
                    let library_addr = library_str.split(':').collect::<Vec<&str>>()[2];
                    add_to_table(&mut deployment_table, broadcast, library_addr, library_name);
                }
                None => eprintln!("Expected an array of libraries."),
            }
        }
    }

    // Loop over the "returns" object.
    if let Some(returned_obj) = json_value.get("returns").and_then(|v| v.as_object()) {
        for (_, value) in returned_obj {
            // Check for the "internal_type" object which should be of the format "contract CONTRACT_NAME"
            if let Some(contract_name) = value.get("internal_type").and_then(|v| v.as_str()) {
                let internal_type_value: Vec<&str> = contract_name.split_whitespace().collect();
                if let Some(contract_name) = internal_type_value.last() {
                    // If the contract name is found, look for contract address
                    if let Some(contract_addr) = value.get("value").and_then(|v| v.as_str()) {
                        // Format the dta and push it to the table
                        add_to_table(&mut deployment_table, broadcast, contract_addr, contract_name);
                    } else {
                        eprintln!("Expected 'value' key");
                    }
                }
            } else {
                eprintln!("Expected 'internal_type' key");
            }
        }
    }

    // Add a newline to separate the tables for different chain ids
    deployment_table.push('\n');

    // Return the deployment table
    deployment_table
}

// Function that reads the TOML chain configurations and extracts them
fn get_all_chains() -> Vec<String> {
    // Define the path to the TOML file
    let toml_path = Path::new("foundry.toml");

    // Read and parse the TOML file content
    let toml_content = match fs::read_to_string(toml_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read the TOML file\n");
            return Vec::new();
        }
    };

    let toml_values: TomlValue = match toml::from_str(&toml_content) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Failed to parse TOML content\n");
            return Vec::new();
        }
    };

    // Extract chains from the TOML data
    let sections = ["rpc_endpoints"];
    let mut chains = Vec::new();

    for section in &sections {
        if let Some(table) = toml_values.get(section).and_then(|v| v.as_table()) {
            chains.extend(table.keys().filter(|&key| key != "localhost").cloned());
        }
    }

    chains.into_iter().collect()
}

// Read broadcast file
fn read_broadcast_file(
    chain_id: &str,
    is_broadcast_deployment: bool,
    script_name: &str,
) -> String {
    if is_broadcast_deployment {
        format!("broadcast/{}/{}/run-latest.json", script_name, chain_id)
    } else {
        format!("broadcast/{}/{}/dry-run/run-latest.json", script_name, chain_id)
    }
}
