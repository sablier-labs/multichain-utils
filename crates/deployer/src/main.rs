use std::{env, fs, io::Write, path::Path, process::Command, thread, time::Duration};
use toml::Value as TomlValue;

mod utils;
use utils::{constants, verify, Broadcast};

fn main() {
    // Process command-line arguments
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter().skip(1);

    // Variables to store flags and provided chains
    let mut broadcast_deployment = false;
    let mut cp_broadcasted_file = false;
    let mut gas_price = "".to_string();
    let mut log_broadcasts = false;
    let mut on_all_chains = false;
    let mut provided_chains = Vec::new();
    let mut sender = "".to_string();
    let mut script_name = "".to_string();
    let mut show_cli = false;
    let mut verify_deployment = false;

    // Parse all arguments
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" => {
                display_message();
                return;
            }
            "--all" => on_all_chains = true,
            "--broadcast" => broadcast_deployment = true,
            "--cp-bf" => cp_broadcasted_file = true,
            "--gas-price" => {
                let value = iter.next().expect("gas price value").to_string();
                gas_price = value.to_string();
            }
            "--log" => log_broadcasts = true,
            "--script" => {
                script_name = iter.next().expect("script name").to_string();
            }
            "--sender" => {
                let sender_address = iter.next().expect("sender address").to_string();
                sender = sender_address.to_string();
            }
            "--show" => show_cli = true,
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

    // Check if a script name was provided
    if script_name.is_empty() {
        println!("No script was provided");
        return;
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

    // Iterate over the provided chains and run the deployment command
    for chain in &provided_chains {
        let env_var = "FOUNDRY_PROFILE=optimized";
        let command = "forge";

        let mut command_args =
            vec!["script".to_string(), format!("script/{}", script_name), "--rpc-url".to_string(), chain.to_string()];

        if broadcast_deployment {
            command_args.push("--broadcast".to_string());
        }

        if !gas_price.is_empty() {
            command_args.push("--gas-price".to_string());
            command_args.push(gas_price.to_string());
        }

        // Push the sender flag.
        command_args.push("--sender".to_string());

        // If no sender address was passed, use the default one.
        if sender.is_empty() {
            sender = constants::DEFAULT_DEPLOYER.to_string();
        }

        // Push the sender address.
        command_args.push(sender.to_string());

        // Add the legacy flag for the "chiliz", "form, and "linea" chains, due to the lack of EIP-3855 support.
        if chain.eq("chiliz") || chain.eq("form") || chain.eq("linea") {
            command_args.push("--legacy".to_string());
        }

        let full_command = format!("{} {} {}", env_var, command, command_args.join(" "));

        if show_cli {
            println!("Command to be executed: {} \n", full_command);
        } else {
            println!("Running the deployment command: {}", full_command);

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

            // Initialize the `Broadcast` instance
            let broadcast = Broadcast::new(&output_str, &script_name, broadcast_deployment)
                .expect("Failed to create Broadcast instance");

            if cp_broadcasted_file {
                broadcast.copy_broadcast_file(chain);
            }

            if log_broadcasts {
                let deployment_table = broadcast.generate_deployment_table();

                // Append the deployment table to the file
                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open("deployments.md")
                    .expect("Failed to open deployment file");
                file.write_all(deployment_table.as_bytes()).expect("Failed to write to the deployment file");
            }
        }
    }

    // If the verify flag is set, run the verification process
    if verify_deployment {
        if !show_cli {
            println!("Waiting for 10 seconds to allow explorer to process deployments... \n");
            thread::sleep(Duration::from_secs(10)); // Sleep for 10 seconds
        }
        verify::verify_contracts(&script_name, &provided_chains, show_cli);
    }
}

// Function to display the help message
pub fn display_message() {
    println!("{}", constants::HELP_MESSAGE);
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
