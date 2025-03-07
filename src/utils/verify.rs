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
struct TransactionData {
    contract_name: Option<String>,
    contract_address: Option<String>,
    arguments: Option<Vec<Value>>,
}

pub fn verify_contracts(
    script_name: &str,
    chains: &Vec<String>,
) {
    let mut txs_data: Vec<(String, TransactionData)> = Vec::new();

    for chain in chains {
        if let Err(e) = process_chain(script_name, chain, &mut txs_data) {
            eprintln!("Error processing chain {}: {}", chain, e);
        }
    }

    // Set environment variable for Foundry profile once
    env::set_var("FOUNDRY_PROFILE", "optimized");

    // Iterate over all transactions and verify each contract.
    for (chain, tx) in txs_data {
        let contract_name = match &tx.contract_name {
            Some(name) => name,
            None => continue,
        };

        let contract_addr = match &tx.contract_address {
            Some(addr) => addr,
            None => continue,
        };

        let constructor_args = if let Some(args) = &tx.arguments {
            if !args.is_empty() {
                match abi_encode(args) {
                    Ok(encoded) => encoded,
                    Err(e) => {
                        eprintln!("Error encoding arguments for {}: {}", contract_name, e);
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

        let mut verifier_flags = get_verifier_flags(&chain);
        args_vec.append(&mut verifier_flags);

        match Command::new("forge").args(&args_vec).output() {
            Ok(output) => {
                if !output.status.success() {
                    println!("Error verifying contract {}: {}", contract_name, String::from_utf8_lossy(&output.stderr));
                } else {
                    println!("Successfully verified {}: {}", contract_name, String::from_utf8_lossy(&output.stdout));
                }
            }
            Err(e) => {
                eprintln!("Failed to run forge verify-contract: {}", e);
            }
        }
    }
}

fn process_chain(
    script_name: &str,
    chain: &str,
    txs_data: &mut Vec<(String, TransactionData)>,
) -> Result<(), String> {
    let chain_id = chain_data::get_chain_id(chain);
    let file_path = format!("broadcast/{}/{}/run-latest.json", script_name, chain_id);

    let json_content =
        fs::read_to_string(&file_path).map_err(|_| format!("Failed to read the broadcast file: {}", &file_path))?;
    let json_value: Value =
        serde_json::from_str(&json_content).map_err(|_| format!("Failed to parse JSON in file: {}", &file_path))?;

    if let Some(tx_array) = json_value.get("transactions").and_then(|v| v.as_array()) {
        for tx_value in tx_array {
            let contract_name = tx_value.get("contractName").and_then(|v| v.as_str()).map(String::from);
            let contract_address = tx_value.get("contractAddress").and_then(|v| v.as_str()).map(String::from);

            let arguments = match tx_value.get("arguments") {
                Some(Value::Array(args)) if !args.is_empty() => Some(args.clone()),
                _ => None,
            };

            txs_data.push((chain.to_string(), TransactionData { contract_name, contract_address, arguments }));
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
