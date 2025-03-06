// FOUNDRY_PROFILE=optimized forge verify-contract 0xCe43436741396Aca6Bf2cB51aD7b35D9474e682e src/Contract.sol:Contract
// --rpc-url sepolia --etherscan-api-key $SEPOLIA_API_KEY
// const CHAINS: &[(&str, &str)] = &[];

use ethabi::{encode, Token};
use hex::encode as hex_encode;
use serde::Deserialize;
use serde_json::Value;
use std::{env, fs, process::Command};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Transaction {
    contract_name: Option<String>,
    contract_address: Option<String>,
    arguments: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
struct Broadcast {
    transactions: Vec<Transaction>,
}

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

fn verify_contracts(
    script_name: &str,
    chain: &str,
    chain_id: &str,
) -> Result<(), String> {
    // Set environment variable for Foundry profile once
    env::set_var("FOUNDRY_PROFILE", "optimized");

    let file_path = format!("broadcast/{}/{}/run-latest.json", script_name, chain_id);
    let file_content = fs::read_to_string(&file_path).map_err(|e| format!("Failed to read broadcast file: {}", e))?;
    let broadcast: Broadcast =
        serde_json::from_str(&file_content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    for tx in &broadcast.transactions {
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
                abi_encode(args)?
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
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

        args_vec.push("--etherscan-api-key".to_string());
        args_vec.push(format!("${}_API_KEY", chain.to_uppercase()));

        let output = Command::new("forge")
            .args(&args_vec)
            .output()
            .map_err(|e| format!("Failed to run forge verify-contract: {}", e))?;

        if !output.status.success() {
            eprintln!("Error verifying contract {}: {}", contract_name, String::from_utf8_lossy(&output.stderr));
        } else {
            println!("Successfully verified {}: {}", contract_name, String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}
