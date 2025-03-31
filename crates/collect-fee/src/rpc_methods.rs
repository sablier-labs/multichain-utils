use crate::constants::{MAX_RETRIES, RETRY_DELAY_SECS};
use alloy::{primitives, providers::Provider, rpc::json_rpc, transports::TransportErrorKind::HttpError};
use http::StatusCode;
use tokio::time::{sleep, Duration};

pub async fn fetch_balance_with_retry<T: Provider>(
    provider: &T,
    eth_address: primitives::Address,
) -> Result<primitives::U256, Box<dyn std::error::Error>> {
    for attempt in 1..=MAX_RETRIES {
        match provider.get_balance(eth_address).await {
            Ok(amount) => return Ok(amount),
            Err(err) => match err {
                json_rpc::RpcError::Transport(HttpError(http_err)) => {
                    if http_err.status == StatusCode::TOO_MANY_REQUESTS {
                        if attempt == MAX_RETRIES {
                            return Err(format!("Rate limit hit after {} requests", MAX_RETRIES).into());
                        }
                        sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                        continue;
                    }
                }
                _ => {
                    continue;
                }
            },
        }
    }

    // If all retries fail, return an error
    Err(format!("Failed to fetch balance for {} after {} retries", eth_address, MAX_RETRIES).into())
}

pub async fn get_chain_id_with_retry<T: Provider>(provider: &T) -> Result<String, Box<dyn std::error::Error>> {
    for attempt in 1..=MAX_RETRIES {
        match provider.get_chain_id().await {
            Ok(chain_id) => return Ok(chain_id.to_string()),
            Err(err) => match err {
                json_rpc::RpcError::Transport(HttpError(http_err)) => {
                    if http_err.status == StatusCode::TOO_MANY_REQUESTS {
                        if attempt == MAX_RETRIES {
                            return Err(format!("Rate limit hit after {} requests", MAX_RETRIES).into());
                        }
                        sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                        continue;
                    }
                }
                _ => {
                    continue;
                }
            },
        }
    }

    // If all retries fail, return an error
    Err(format!("Failed to fetch chain id after {} retries", MAX_RETRIES).into())
}
