use alloy::providers;

mod collect_fees;
mod constants;
mod rpc_methods;
mod states;

//////////////////////////////////////////////////////////////////////////
//                                 MAIN
//////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // Check `PRIVATE_KEY` exists in .env file.
    let private_key_env = match std::env::var("PRIVATE_KEY") {
        Ok(pk) => pk,
        Err(_) => {
            return Err("Missing PRIVATE_KEY".into());
        }
    };

    let eoa_wallet = states::EOAWallet::new(private_key_env);

    let deployments = states::Deployments::load_from_file(constants::DATA_FILE_DEPLOYED_CONTRACTS)?;
    for (chain_name, deployed_contracts) in deployments.chains {
        // Check `{CHAIN_NAME}_RPC_URL` exists in .env file.
        let rpc_url_key = format!("{}_RPC_URL", chain_name.to_uppercase());
        let rpc_url = match std::env::var(&rpc_url_key) {
            Ok(url) => url,
            Err(_) => {
                println!("Missing {}", rpc_url_key);
                continue;
            }
        };

        let provider = providers::ProviderBuilder::new().wallet(&eoa_wallet.signer).on_http(rpc_url.parse()?);

        // Check if the eoa has enough ether to pay for gas.
        let eoa_balance = match rpc_methods::fetch_balance_with_retry(&provider, eoa_wallet.address).await {
            Ok(amount) => amount,
            Err(_) => {
                println!("Invalid RPC endpoint for {}", chain_name);
                continue;
            }
        };
        if eoa_balance.is_zero() {
            println!("Insufficient EOA balance on {}", chain_name);
            continue;
        }

        // Collect fees from all contracts on this chain.
        match collect_fees::collect_fees_on_chain(&provider, &deployed_contracts, &chain_name).await {
            Ok(_) => {}
            Err(_) => {
                continue;
            }
        }
    }

    Ok(())
}
