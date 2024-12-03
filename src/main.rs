use serde_json::Value;
use std::{env, fs, path::Path, process::Command};
use toml::Value as TomlValue;

fn main() {
    // Process command-line arguments
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter().skip(1);

    // Variables to store flags and provided chains
    let mut broadcast_deployment = "".to_string();
    let mut verify_deployment = false;
    let mut cp_broadcasted_file = false;
    let mut gas_price = "".to_string();
    let mut script_name = "".to_string();
    let mut on_all_chains = false;
    let mut provided_chains = Vec::new();

    // Parse all arguments
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--all" => on_all_chains = true,
            "--cp-bf" => cp_broadcasted_file = true,
            "--script" => {
                script_name = iter.next().expect("script name").to_string();
            }
            "--broadcast" => broadcast_deployment = "--broadcast".to_string(),
            "--verify" => verify_deployment = true,
            "--gas-price" => {
                let value = iter.next().expect("gas price value").to_string();
                gas_price = format!(" --gas-price {}", value);
            }
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
        println!("No script name provided, using default script: {}", script_name);
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
                println!("Chain {} is not configured in the TOML file", chain);
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
    println!("Deploying to the chains: {}", chains_string);

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

        println!("Running the deployment command: {} {} {}", env_var, command, command_args.join(" "));

        // Set the environment variable
        let env_var_parts: Vec<&str> = env_var.split('=').collect();
        env::set_var(env_var_parts[0], env_var_parts[1]);

        // Create the CLI and capture the command output
        let output = Command::new(command).args(&command_args).output().expect("Failed to run command");

        // Process command output
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output.status.success() {
            println!("Command output: {}", output_str);
        } else {
            eprintln!("Command failed with error: {}", String::from_utf8_lossy(&output.stderr));
        }

        // Move broadcast file if needed
        if cp_broadcasted_file {
            move_broadcast_file(&script_name, &chain, &output_str, !broadcast_deployment.is_empty());
        }
    }
}

// Function that reads the TOML chain configurations and extracts them
fn get_all_chains() -> Vec<String> {
    // Define the path to the TOML file
    let toml_path = Path::new("foundry.toml");

    // Read and parse the TOML file content
    let toml_content = match fs::read_to_string(toml_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read the TOML file");
            return Vec::new();
        }
    };

    let toml_values: TomlValue = match toml::from_str(&toml_content) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Failed to parse TOML content");
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

fn move_broadcast_file(script_name: &str, chain: &str, output: &str, is_broadcast_deployment: bool) {
    let project = if script_name.starts_with("Protocol") || script_name.ends_with("Protocol") {
        "lockup".to_string()
    } else if script_name.contains("Flow") {
        "flow".to_string()
    } else if script_name.contains("MerkleFactory") {
        "airdrops".to_string()
    } else {
        // skip this function if the script name doesn't match any of the above
        return;
    };

    // Extract the chain_id from the output
    let chain_id = output
        .lines()
        .find(|line| line.trim().starts_with("Chain "))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("");

    let broadcast_file_path = if is_broadcast_deployment {
        format!("broadcast/{}/{}/run-latest.json", script_name, chain_id)
    } else {
        format!("broadcast/{}/{}/dry-run/run-latest.json", script_name, chain_id)
    };

    let version = serde_json::from_str::<Value>(&fs::read_to_string("package.json").unwrap()).unwrap()["version"]
        .as_str()
        .unwrap()
        .to_string();

    let dest_path = format!("../v2-deployments/{}/v{}/broadcasts/{}.json", project, version, chain);

    // Create the parent directory if it doesn't exist
    if let Some(parent) = Path::new(&dest_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
    }

    // Move and rename the file
    fs::rename(&broadcast_file_path, &dest_path).expect("Failed to move and rename run-latest.json to v2-deployments");
}
