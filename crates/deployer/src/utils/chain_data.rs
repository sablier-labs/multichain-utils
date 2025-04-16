pub struct ChainData {
    pub explorer_url: &'static str,
    pub name: &'static str,
}

const CHAINS: &[(&str, ChainData)] = &[
    // Mainnets
    ("1", ChainData { name: "Ethereum Mainnet", explorer_url: "https://etherscan.io/" }),
    ("2741", ChainData { name: "Abstract", explorer_url: "https://abscan.org/" }),
    ("42161", ChainData { name: "Arbitrum One", explorer_url: "https://arbiscan.io/" }),
    ("43114", ChainData { name: "Avalanche", explorer_url: "https://snowtrace.io/" }),
    ("8453", ChainData { name: "Base", explorer_url: "https://basescan.org/" }),
    ("80094", ChainData { name: "Berachain", explorer_url: "https://berascan.com/" }),
    ("81457", ChainData { name: "Blast", explorer_url: "https://blastscan.io/" }),
    ("56", ChainData { name: "BNB Smart Chain", explorer_url: "https://bscscan.com/" }),
    ("88888", ChainData { name: "Chiliz", explorer_url: "https://chiliscan.com/" }),
    ("1116", ChainData { name: "Core Dao", explorer_url: "https://scan.coredao.org/" }),
    ("478", ChainData { name: "Form", explorer_url: "https://explorer.form.network/" }),
    ("100", ChainData { name: "Gnosis", explorer_url: "https://gnosisscan.io/" }),
    ("4689", ChainData { name: "IoTex", explorer_url: "https://iotexscan.io/" }),
    ("1890", ChainData { name: "Lightlink", explorer_url: "https://phoenix.lightlink.io/" }),
    ("59144", ChainData { name: "Linea", explorer_url: "https://lineascan.build/" }),
    ("333000333", ChainData { name: "Meld", explorer_url: "https://meldscan.io/" }),
    ("34443", ChainData { name: "Mode", explorer_url: "https://explorer.mode.network/" }),
    ("2818", ChainData { name: "Morph", explorer_url: "https://explorer.morphl2.io/" }),
    ("10", ChainData { name: "Optimism", explorer_url: "https://optimistic.etherscan.io/" }),
    ("137", ChainData { name: "Polygon", explorer_url: "https://polygonscan.com/" }),
    ("534352", ChainData { name: "Scroll", explorer_url: "https://scrollscan.com/" }),
    ("5330", ChainData { name: "Superseed", explorer_url: "https://explorer.superseed.xyz/" }),
    ("167000", ChainData { name: "Taiko Mainnet", explorer_url: "https://taikoscan.io/" }),
    ("5845", ChainData { name: "Tangle", explorer_url: "https://explorer.tangle.tools/" }),
    ("50", ChainData { name: "XDC", explorer_url: "https://xdcscan.com/" }),
    ("324", ChainData { name: "zkSync Era", explorer_url: "https://era.zksync.network/" }),
    // Testnets
    ("11155111", ChainData { name: "Sepolia", explorer_url: "https://sepolia.etherscan.io/" }),
    ("421614", ChainData { name: "Arbitrum Sepolia", explorer_url: "https://sepolia.arbiscan.io/" }),
    ("84532", ChainData { name: "Base Sepolia", explorer_url: "https://sepolia.basescan.org/" }),
    ("80084", ChainData { name: "Berachain Bartio", explorer_url: "https://sepolia.berachain.io/" }),
    ("168587773", ChainData { name: "Blast Sepolia", explorer_url: "https://sepolia.blastscan.io/" }),
    ("59141", ChainData { name: "Linea Sepolia", explorer_url: "https://sepolia.lineascan.build/" }),
    ("919", ChainData { name: "Mode Sepolia", explorer_url: "https://sepolia.explorer.mode.network/" }),
    ("10143", ChainData { name: "Monad Testnet", explorer_url: "https://testnet.monadexplorer.com/" }),
    ("2810", ChainData { name: "Morph Holesky", explorer_url: "https://sepolia.morphl2.io/" }),
    ("11155420", ChainData { name: "Optimism Sepolia", explorer_url: "https://sepolia-optimism.etherscan.io/" }),
    ("974399131", ChainData { name: "SKALE Testnet", explorer_url: "https://sepolia.skale.io/" }),
    ("53302", ChainData { name: "Superseed Sepolia", explorer_url: "sepolia-explorer.superseed.xyz/" }),
    ("167009", ChainData { name: "Taiko Hekla", explorer_url: "https://hekla.taikoexplorer.com/" }),
    ("300", ChainData { name: "zkSync Sepolia", explorer_url: "https://sepolia-era.zksync.network/" }),
];

/// Returns the chain id for a given partial or complete chain name.
/// The search is performed in a case-insensitive manner. If a match is found (i.e. the chain's
/// name contains the provided query), the corresponding chain id is returned.
/// Otherwise, it returns the provided chain_name.
pub fn get_chain_id(chain_name: &str) -> &str {
    let query = chain_name.to_lowercase();
    CHAINS
        .iter()
        .find(|(_, data)| data.name.to_lowercase().contains(&query))
        .map(|(chain_id, _)| *chain_id)
        .unwrap_or(chain_name)
}

/// Returns the chain data for a given chain_id, if it exists.
pub fn get_data(chain_id: &str) -> Option<&'static ChainData> {
    CHAINS.iter().find(|entry| entry.0 == chain_id).map(|entry| &entry.1)
}

/// Returns the explorer URL for a given chain_id.
/// If the chain_id is not found, returns "<N/A>".
pub fn get_explorer_url(chain_id: &str) -> String {
    get_data(chain_id).map(|data| data.explorer_url.to_string()).unwrap_or_else(|| "<N/A>".to_string())
}

/// Returns the explorer URL based on a partial or complete chain name.
/// The search is performed in a case-insensitive manner by converting both the input
/// and each chain's name to lowercase. If a match is found (i.e. the chain's name contains
/// the provided query), the corresponding explorer URL is returned. Otherwise, it returns "<N/A>".
pub fn get_explorer_url_by_name(chain_name: &str) -> String {
    let query = chain_name.to_lowercase();
    CHAINS
        .iter()
        .find(|(_, data)| data.name.to_lowercase().contains(&query))
        .map(|(_, data)| data.explorer_url.to_string())
        .unwrap_or_else(|| "<N/A>".to_string())
}

/// Returns the explorer URL for a given chain_id and contract_addr.
/// This function first obtains the base explorer URL using `get_explorer_url`
/// and then appends the path "address/" followed by the contract address.
/// If the chain_id is not found (i.e. the base URL is "<N/A>"), it returns "<N/A>".
pub fn get_explorer_url_with_address(
    chain_id: &str,
    contract_addr: &str,
) -> String {
    let base_url = get_explorer_url(chain_id);
    if base_url == "<N/A>" {
        "<N/A>".to_string()
    } else {
        format!("{}address/{}", base_url, contract_addr)
    }
}

/// Returns the chain name for a given chain_id.
/// If the chain_id is not found, returns the chain_id itself.
pub fn get_name(chain_id: &str) -> &str {
    get_data(chain_id).map(|data| data.name).unwrap_or(chain_id)
}
