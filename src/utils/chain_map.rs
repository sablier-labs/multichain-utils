pub fn chain_name(chain_id: &str) -> &str {
    match chain_id {
        // Mainnets
        "1" => "Ethereum Mainnet",
        "2741" => "Abstract",
        "42161" => "Arbitrum One",
        "43114" => "Avalanche",
        "8453" => "Base",
        "80094" => "Berachain",
        "81457" => "Blast",
        "56" => "BNB Smart Chain",
        "88888" => "Chiliz",
        "1116" => "Core Dao",
        "478" => "Form",
        "100" => "Gnosis",
        "4689" => "IoTex",
        "1890" => "Lightlink",
        "59144" => "Linea",
        "333000333" => "Meld",
        "34443" => "Mode",
        "2818" => "Morph",
        "10" => "Optimism",
        "137" => "Polygon",
        "534352" => "Scroll",
        "5330" => "Superseed",
        "167000" => "Taiko Mainnet",
        "5845" => "Tangle",
        "50" => "XDC",
        "324" => "zkSync Era",
        // Testnets
        "11155111" => "Sepolia",
        "421614" => "Arbitrum Sepolia",
        "84532" => "Base Sepolia",
        "80084" => "Berachain Bartio",
        "168587773" => "Blast Sepolia",
        "59141" => "Linea Sepolia",
        "919" => "Mode Sepolia",
        "10143" => "Monad Testnet",
        "2810" => "Morph Holesky",
        "11155420" => "Optimism Sepolia",
        "974399131" => "SKALE Testnet",
        "53302" => "Superseed Sepolia",
        "167009" => "Taiko Hekla",
        "300" => "zkSync Sepolia",
        // Return chain_id if no match found
        _ => chain_id,
    }
}

pub fn explorer_url(
    chain_id: &str,
    contract_addr: &str,
) -> String {
    let base_url = match chain_id {
        // Mainnets
        "1" => "https://etherscan.io/address/",
        "2741" => "https://wwstage.abscan.org/address/",
        "42161" => "https://arbiscan.io/address/",
        "43114" => "https://snowtrace.io/address/",
        "8453" => "https://basescan.org/address/",
        "80094" => "https://berascan.com/address/",
        "81457" => "https://blastscan.io/address/",
        "56" => "https://bscscan.com/address/",
        "88888" => "https://chiliscan.com/address/",
        "1116" => "https://scan.coredao.org/address/",
        "478" => "https://explorer.form.network/address/",
        "100" => "https://gnosisscan.io/address/",
        "4689" => "https://iotexscan.io/address/",
        "1890" => "https://phoenix.lightlink.io/address/",
        "59144" => "https://lineascan.build/address/",
        "333000333" => "https://meldscan.io/address/",
        "34443" => "https://explorer.mode.network/address/",
        "2818" => "https://explorer.morphl2.io/address/",
        "10" => "https://optimistic.etherscan.io/address/",
        "137" => "https://polygonscan.com/address/",
        "534352" => "https://scrollscan.com/address/",
        "5330" => "https://explorer.superseed.xyz/address/",
        "167000" => "https://taikoscan.io/address/",
        "5845" => "https://explorer.tangle.tools/address/",
        "50" => "https://xdcscan.com/address/",
        "324" => "https://era.zksync.network/address/",
        // Testnets
        "11155111" => "https://sepolia.etherscan.io/address/",
        "421614" => "https://sepolia.arbiscan.io/address/",
        "84532" => "https://sepolia.basescan.org/address/",
        "80084" => "https://sepolia.berachain.io/address/",
        "168587773" => "https://sepolia.blastscan.io/address/",
        "59141" => "https://sepolia.lineascan.build/address/",
        "919" => "https://sepolia.explorer.mode.network/address/",
        "10143" => "https://testnet.monadexplorer.com/address/",
        "2810" => "https://sepolia.morphl2.io/address/",
        "11155420" => "https://sepolia.optimistic.etherscan.io/address/",
        "974399131" => "https://sepolia.skale.io/address/",
        "53302" => "https://sepolia.superseed.xyz/address/",
        "167009" => "https://sepolia.taikoscan.io/address/",
        "300" => "https://sepolia.zksync.network/address/",
        // Return a placeholder if no match found
        _ => "<N/A>",
    };

    if base_url == "<N/A>" {
        base_url.to_string()
    } else {
        format!("{}{}", base_url, contract_addr)
    }
}
