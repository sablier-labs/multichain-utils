pub struct ChainData {
    pub explorer_url: &'static str,
    pub name: &'static str,
}

#[rustfmt::skip]
const CHAINS: &[(&str, ChainData)] = &[
    // Mainnets
    ("1", ChainData { name: "Ethereum Mainnet", explorer_url: "https://etherscan.io/address/" }),
    ("2741", ChainData { name: "Abstract", explorer_url: "https://abscan.org/address/" }),
    ("42161", ChainData { name: "Arbitrum One", explorer_url: "https://arbiscan.io/address/" }),
    ("43114", ChainData { name: "Avalanche", explorer_url: "https://snowtrace.io/address/" }),
    ("8453", ChainData { name: "Base", explorer_url: "https://basescan.org/address/" }),
    ("80094", ChainData { name: "Berachain", explorer_url: "https://berascan.com/address/" }),
    ("81457", ChainData { name: "Blast", explorer_url: "https://blastscan.io/address/" }),
    ("56", ChainData { name: "BNB Smart Chain", explorer_url: "https://bscscan.com/address/" }),
    ("88888", ChainData { name: "Chiliz", explorer_url: "https://chiliscan.com/address/" }),
    ("1116", ChainData { name: "Core Dao", explorer_url: "https://scan.coredao.org/address/" }),
    ("478", ChainData { name: "Form", explorer_url: "https://explorer.form.network/address/" }),
    ("100", ChainData { name: "Gnosis", explorer_url: "https://gnosisscan.io/address/" }),
    ("4689", ChainData { name: "IoTex", explorer_url: "https://iotexscan.io/address/" }),
    ("1890", ChainData { name: "Lightlink", explorer_url: "https://phoenix.lightlink.io/address/" }),
    ("59144", ChainData { name: "Linea", explorer_url: "https://lineascan.build/address/" }),
    ("333000333", ChainData { name: "Meld", explorer_url: "https://meldscan.io/address/" }),
    ("34443", ChainData { name: "Mode", explorer_url: "https://explorer.mode.network/address/" }),
    ("2818", ChainData { name: "Morph", explorer_url: "https://explorer.morphl2.io/address/" }),
    ("10", ChainData { name: "Optimism", explorer_url: "https://optimistic.etherscan.io/address/" }),
    ("137", ChainData { name: "Polygon", explorer_url: "https://polygonscan.com/address/" }),
    ("534352", ChainData { name: "Scroll", explorer_url: "https://scrollscan.com/address/" }),
    ("5330", ChainData { name: "Superseed", explorer_url: "https://explorer.superseed.xyz/address/" }),
    ("167000", ChainData { name: "Taiko Mainnet", explorer_url: "https://taikoscan.io/address/" }),
    ("5845", ChainData { name: "Tangle", explorer_url: "https://explorer.tangle.tools/address/" }),
    ("50", ChainData { name: "XDC", explorer_url: "https://xdcscan.com/address/" }),
    ("324", ChainData { name: "zkSync Era", explorer_url: "https://era.zksync.network/address/" }),
    // Testnets
    ("11155111", ChainData { name: "Sepolia", explorer_url: "https://sepolia.etherscan.io/address/" }),
    ("421614", ChainData { name: "Arbitrum Sepolia", explorer_url: "https://sepolia.arbiscan.io/address/" }),
    ("84532", ChainData { name: "Base Sepolia", explorer_url: "https://sepolia.basescan.org/address/" }),
    ("80084", ChainData { name: "Berachain Bartio", explorer_url: "https://sepolia.berachain.io/address/" }),
    ("168587773", ChainData { name: "Blast Sepolia", explorer_url: "https://sepolia.blastscan.io/address/" }),
    ("59141", ChainData { name: "Linea Sepolia", explorer_url: "https://sepolia.lineascan.build/address/" }),
    ("919", ChainData { name: "Mode Sepolia", explorer_url: "https://sepolia.explorer.mode.network/address/" }),
    ("10143", ChainData { name: "Monad Testnet", explorer_url: "https://testnet.monadexplorer.com/address/" }),
    ("2810", ChainData { name: "Morph Holesky", explorer_url: "https://sepolia.morphl2.io/address/" }),
    ("11155420",ChainData { name: "Optimism Sepolia", explorer_url: "https://sepolia.optimistic.etherscan.io/address/" }),
    ("974399131", ChainData { name: "SKALE Testnet", explorer_url: "https://sepolia.skale.io/address/" }),
    ("53302", ChainData { name: "Superseed Sepolia", explorer_url: "https://sepolia.superseed.xyz/address/" }),
    ("167009", ChainData { name: "Taiko Hekla", explorer_url: "https://sepolia.taikoscan.io/address/" }),
    ("300", ChainData { name: "zkSync Sepolia", explorer_url: "https://sepolia.zksync.network/address/" }),
];

/// Returns the chain data for a given chain_id, if it exists.
pub fn get_data(chain_id: &str) -> Option<&'static ChainData> {
    CHAINS.iter().find(|entry| entry.0 == chain_id).map(|entry| &entry.1)
}

/// Returns the explorer URL for a given chain_id.
/// If the chain_id is not found, returns "<N/A>".
pub fn get_explorer(
    chain_id: &str,
    contract_addr: &str,
) -> String {
    get_data(chain_id)
        .map(|data| format!("{}{}", data.explorer_url, contract_addr))
        .unwrap_or_else(|| "<N/A>".to_string())
}

/// Returns the chain name for a given chain_id.
/// If the chain_id is not found, returns the chain_id itself.
pub fn get_name(chain_id: &str) -> &str {
    get_data(chain_id).map(|data| data.name).unwrap_or(chain_id)
}
