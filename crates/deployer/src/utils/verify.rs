use super::chain_data;
use ethabi::{encode, Token};
use hex::encode as hex_encode;
use serde_json::Value;
use std::{env, fs, process::Command};

fn abi_encode(args: &[Value]) -> Result<String, String> {
    let tokens: Result<Vec<Token>, String> = args
        .iter()
        .map(|arg| {
            if let Some(addr) = arg.as_str() {
                addr.parse().map(Token::Address).map_err(|e| format!("Failed to parse address {}: {}", addr, e))
            } else if let Some(num) = arg.as_u64() {
                Ok(Token::Uint(num.into()))
            } else {
                Err("Unsupported constructor argument type".to_string())
            }
        })
        .collect();

    let tokens = tokens?;
    let abi_encoded_value = encode(&tokens);
    Ok(format!("0x{}", hex_encode(abi_encoded_value)))
}

#[derive(Debug)]
struct VerifyData {
    contract_name: Option<String>,
    contract_address: Option<String>,
    arguments: Option<Vec<Value>>,
    libraries: Option<Vec<String>>,
}

pub fn verify_contracts(
    script_name: &str,
    chains: &Vec<String>,
    show_cli: bool,
) {
    let mut verify_data: Vec<(String, VerifyData)> = Vec::new();

    for chain in chains {
        if let Err(e) = process_chain(script_name, chain, &mut verify_data, show_cli) {
            println!("Error verifying chain {}: {}", chain, e);
        }
    }

    // Set environment variable for Foundry profile once
    env::set_var("FOUNDRY_PROFILE", "optimized");

    // Iterate over all transactions and verify each contract.
    for (chain, data) in verify_data {
        let contract_name = match &data.contract_name {
            Some(name) => name,
            None => continue,
        };

        let contract_addr = match &data.contract_address {
            Some(addr) => addr,
            None => continue,
        };

        let constructor_args = if let Some(args) = &data.arguments {
            if !args.is_empty() {
                match abi_encode(args) {
                    Ok(encoded) => encoded,
                    Err(e) => {
                        println!("For chain {}, error encoding arguments for {}: {}", chain, contract_name, e);
                        continue;
                    }
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut args_vec = vec![
            "verify-contract".to_string(),
            contract_addr.to_string(),
            format!("src/{}.sol:{}", contract_name, contract_name),
        ];

        if !constructor_args.is_empty() {
            args_vec.push("--constructor-args".to_string());
            args_vec.push(constructor_args);
        }

        // Add libraries if any
        if let Some(libraries) = &data.libraries {
            for lib in libraries {
                if !lib.is_empty() {
                    args_vec.push("--libraries".to_string());
                    args_vec.push(lib.clone());
                }
            }
        }

        let mut verifier_flags = get_verifier_flags(&chain);
        args_vec.append(&mut verifier_flags);

        let full_command = format!("forge {}", args_vec.join(" "));

        if show_cli {
            println!("Verification command to be executed: FOUNDRY_PROFILE=optimized {} \n", full_command);
        } else {
            match Command::new("forge").args(&args_vec).output() {
                Ok(output) => {
                    if !output.status.success() {
                        println!(
                            "For chain {}, the verification did not work for contract {} using the command: \
                             {}\nError: {}",
                            chain,
                            contract_name,
                            full_command,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    } else {
                        println!(
                            "Successfully verified {}: {}",
                            contract_name,
                            String::from_utf8_lossy(&output.stdout)
                        );
                    }
                }
                Err(e) => {
                    println!("For chain {}, failed to run forge verify-contract command: {}", chain, e);
                }
            }
        }
    }
}

fn process_chain(
    script_name: &str,
    chain: &str,
    verify_data: &mut Vec<(String, VerifyData)>,
    show_cli: bool,
) -> Result<(), String> {
    let chain_id = chain_data::get_chain_id(chain);
    let file_path: String = if show_cli {
        format!("broadcast/{}/{}/dry-run/run-latest.json", script_name, chain_id)
    } else {
        format!("broadcast/{}/{}/run-latest.json", script_name, chain_id)
    };

    let json_content =
        fs::read_to_string(&file_path).map_err(|_| format!("Failed to read the broadcast file: {}", &file_path))?;
    let json_value: Value =
        serde_json::from_str(&json_content).map_err(|_| format!("Failed to parse JSON in file: {}", &file_path))?;

    // Extract libraries from the JSON
    let libraries: Option<Vec<String>> = json_value
        .get("libraries")
        .and_then(|v| v.as_array())
        .map(|lib_array| lib_array.iter().filter_map(|lib| lib.as_str().map(|s| s.to_string())).collect());

    if let Some(tx_array) = json_value.get("transactions").and_then(|v| v.as_array()) {
        for tx_value in tx_array {
            let contract_name = tx_value.get("contractName").and_then(|v| v.as_str()).map(String::from);
            let contract_address = tx_value.get("contractAddress").and_then(|v| v.as_str()).map(String::from);

            let arguments = match tx_value.get("arguments") {
                Some(Value::Array(args)) if !args.is_empty() => Some(args.clone()),
                _ => None,
            };

            verify_data.push((
                chain.to_string(),
                VerifyData { contract_name, contract_address, arguments, libraries: libraries.clone() },
            ));
        }
    }

    Ok(())
}

fn get_verifier_flags(chain: &str) -> Vec<String> {
    let mut args = vec!["--verify".to_string()];
    args.push("--etherscan-api-key".to_string());

    if chain.eq("form") || chain.eq("lightlink") || chain.eq("mode") || chain.eq("morph") || chain.eq("superseed") {
        args.push("\"verifyContract\"".to_string());
        let explorer_url = chain_data::get_explorer_url_by_name(chain);
        args.push("--verifier-url".to_string());
        args.push(format!("{}api\\?", explorer_url));
    } else if chain.eq("chiliz") {
        args.push("\"verifyContract\"".to_string());
        args.push("--verifier-url".to_string());
        args.push("https://api.routescan.io/v2/network/mainnet/evm/88888/etherscan".to_string());
    } else {
        args.push(format!("${}_API_KEY", chain.to_uppercase()));
    }

    args
}
