pub struct ChainData {
    pub explorer_url: &'static str,
    pub name: &'static str,
}

const CHAINS: &[(&str, ChainData)] = &[
    // Mainnets
    ("1", ChainData { name: "ethereum", explorer_url: "https://etherscan.io/" }),
    ("2741", ChainData { name: "abstract", explorer_url: "https://abscan.org/" }),
    ("42161", ChainData { name: "arbitrum", explorer_url: "https://arbiscan.io/" }),
    ("43114", ChainData { name: "avalanche", explorer_url: "https://snowtrace.io/" }),
    ("8453", ChainData { name: "base", explorer_url: "https://basescan.org/" }),
    ("80094", ChainData { name: "berachain", explorer_url: "https://berascan.com/" }),
    ("81457", ChainData { name: "blast", explorer_url: "https://blastscan.io/" }),
    ("56", ChainData { name: "bsc", explorer_url: "https://bscscan.com/" }),
    ("88888", ChainData { name: "chiliz", explorer_url: "https://chiliscan.com/" }),
    ("1116", ChainData { name: "core_dao", explorer_url: "https://scan.coredao.org/" }),
    ("100", ChainData { name: "gnosis", explorer_url: "https://gnosisscan.io/" }),
    ("999", ChainData { name: "hyperevm", explorer_url: "https://hyperevmscan.io/" }),
    ("1890", ChainData { name: "lightlink", explorer_url: "https://phoenix.lightlink.io/" }),
    ("59144", ChainData { name: "linea", explorer_url: "https://lineascan.build/" }),
    ("34443", ChainData { name: "mode", explorer_url: "https://explorer.mode.network/" }),
    ("2818", ChainData { name: "morph", explorer_url: "https://explorer.morphl2.io/" }),
    ("10", ChainData { name: "optimism", explorer_url: "https://optimistic.etherscan.io/" }),
    ("137", ChainData { name: "polygon", explorer_url: "https://polygonscan.com/" }),
    ("534352", ChainData { name: "scroll", explorer_url: "https://scrollscan.com/" }),
    ("1329", ChainData { name: "sei", explorer_url: "https://seiscan.io/" }),
    ("146", ChainData { name: "sonic", explorer_url: "https://sonicscan.org/" }),
    ("50104", ChainData { name: "sophon", explorer_url: "https://explorer.sophon.xyz/" }),
    ("5330", ChainData { name: "superseed", explorer_url: "https://explorer.superseed.xyz/" }),
    ("5845", ChainData { name: "tangle", explorer_url: "https://explorer.tangle.tools/" }),
    ("130", ChainData { name: "unichain", explorer_url: "https://uniscan.xyz/" }),
    ("50", ChainData { name: "xdc", explorer_url: "https://xdcscan.com/" }),
    ("324", ChainData { name: "zksync", explorer_url: "https://era.zksync.network/" }),
    // Testnets
    ("11155111", ChainData { name: "sepolia", explorer_url: "https://sepolia.etherscan.io/" }),
    ("421614", ChainData { name: "arbitrum_sepolia", explorer_url: "https://sepolia.arbiscan.io/" }),
    ("84532", ChainData { name: "base_sepolia", explorer_url: "https://sepolia.basescan.org/" }),
    ("919", ChainData { name: "mode_sepolia", explorer_url: "https://sepolia.explorer.mode.network/" }),
    ("11155420", ChainData { name: "optimism_sepolia", explorer_url: "https://sepolia-optimism.etherscan.io/" }),
];

/// Returns the chain id for a given chain name.
pub fn get_chain_id(chain_name: &str) -> &str {
    CHAINS.iter().find(|(_, c)| c.name == chain_name).unwrap().0
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
