## Config details

### List of chains:

- abstract: 2741
- arbitrum: 42161
- avalanche: 43114
- base: 8453
- berachain: 80094
- blast: 81457
- bsc: 56
- chiliz: 88888
- core_dao: 1116
- ethereum: 1
- gnosis: 100
- hyperevm: 999
- lightlink: 1890
- linea: 59144
- mode: 34443
- morph: 2818
- optimism: 10
- polygon: 137
- scroll: 534352
- sei: 1329
- sonic: 146
- sophon: 50104
- superseed: 5330
- unichain: 130
- xdc: 50
- zksync: 324
- arbitrum_sepolia: 421614
- base_sepolia: 84532
- mode_sepolia: 919
- optimism_sepolia: 11155420
- sepolia: 11155111

### Etherscan-supported chains:

- abstract
- arbitrum
- base
- berachain
- blast
- bsc
- ethereum
- gnosis
- hyperevm
- linea
- optimism
- polygon
- scroll
- sonic
- unichain
- xdc
- zksync
- arbitrum_sepolia
- base_sepolia
- optimism_sepolia
- sepolia

### Blockscout-supported chains:

- avalanche: verifier-url https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan
- chiliz: verifier-url https://api.routescan.io/v2/network/mainnet/evm/88888/
- core_dao: verifier-url https://openapi.coredao.org/api/
- lightlink: https://phoenix.lightlink.io/api/
- mode: verifier-url https://explorer.mode.network/api/
- morph: verifier-url https://explorer-api.morphl2.io/api/
- sei: verifier-url https://sei.explorer.zenscan.io/api/
- sophon: verifier-url https://explorer.sophon.xyz/api/
- superseed: verifier-url https://explorer.superseed.xyz/api/
- mode_sepolia: verifier-url https://sepolia.explorer.mode.network/api/

### Doesn't support latest EIPs, i.e. require `--legacy` flag:

- abstract
- chiliz
- core_dao
- lightlink
- xdc

### Needs extra details

Chilliz chain isn't able to accuratelly estimate the gas price, so we need to provide the flags explicitly:

- `--with-gas-price 10000000000000`
- `--priority-gas-price 1000000000`

# Steps for the script

## First step: deploy command + verification

### Etherscan forge command:

```sh
FOUNDRY_PROFILE=optimized forge script scripts/solidity/<ScriptName>.s.sol:ContractName \
  --rpc-url <chain_name> \
  --broadcast \
  --sig "run()" \
  --sender <wallet> \
  --private-key $PRIVATE_KEY \
  --verify \
  --verifier etherscan \
  --etherscan-api-key $ETHERSCAN_API_KEY
```

### Blockscout forge command:

```sh
FOUNDRY_PROFILE=optimized forge script scripts/solidity/<ScriptName>.s.sol:ContractName \
  --rpc-url <chain_name> \
  --broadcast \
  --sig "run()" \
  --sender <wallet> \
  --private-key $PRIVATE_KEY \
  --verifier blockscout \
  --verifier-url '<blockscout_homepage_explorer_url>/api/' \
  --etherscan-api-key 'verifyContract'
```

## Second step: move broadcast file to sdk

```sh
cp broadcast/<ScriptName>.s.sol/<chain_id>/run-latest.json \
  ../sdk/deployments/<protocol_name>/v<from_package.json>/broadcasts/<chain_name>.json
```

the protocol name and the version should be extracted from the `package.json` file. example:

```json
{
  "name": "@sablier/flow",
  "version": "3.0.0"
}
```

protocol name: `flow`
version: `3.0.0`

## Third step: extract the data from broadcast json

In the broadcast file there are "receipts" fields with multiple data, but it includes the "blockNumber" and "contractAddress". Example:

```json
{
  "receipts": [
    {
      "status": "0x1",
      "cumulativeGasUsed": "0x29779d8",
      "logs": [],
      "logsBloom": "<logs_bloom>",
      "type": "0x2",
      "transactionHash": "0x89a4e564f8e31c8c1d643dc73e94a5f6fea24c6c3941ddefd2d7c451416c9c8c",
      "transactionIndex": "0xde",
      "blockHash": "0x88de7139243df0fe3162d8ebebd4d5c40b0bbde86921fc69b8ac7c96a3245f69",
      "blockNumber": "0x1669749",
      "gasUsed": "0xcb494",
      "effectiveGasPrice": "0x7860ff6",
      "from": "0xb1bef51ebca01eb12001a639bdbbff6eeca12b9f",
      "to": "0x4e59b44847b379578588920ca78fbf26c0b4956c",
      "contractAddress": "0xa0a1ac47260b95d334763473b868117ef7343aa0"
    }
  ]
}
```

notice that the number is in hex, so it needs to be converted to decimal.

also, the broadcast json also contains the "transactions" field, which includes the "contractName" and "contractAddress". Example:

```json
{
  "transactions": [
    {
      "hash": "0x8ac5e7148372a1df22341b0d2d29e8570e99221f3236d2a04abdd28aa4e1335e",
      "transactionType": "CREATE2",
      "contractName": "SablierLockup",
      "contractAddress": "0xcf8ce57fa442ba50acbc57147a62ad03873ffa73",
      "function": null,
      "arguments": [
        "<contract_address>",
        "0xA9dC6878C979B5cc1d98a1803F0664ad725A1f56"
      ],
      "transaction": {
        "from": "0xb1bef51ebca01eb12001a639bdbbff6eeca12b9f",
        "to": "0x4e59b44847b379578588920ca78fbf26c0b4956c",
        "gas": "0x744aef",
        "value": "0x0",
        "input": "<bytecode>",
        "nonce": "0x31",
        "chainId": "0x1"
      },
      "additionalContracts": [],
      "isFixedGasLimit": false
    }
  ]
}
```

the contract names are in camel cases, but i want them in upper snake case, so i need to convert them. Example: `SablierLockup` -> `SABLIER_LOCKUP`

based on this data that can be obtained from the broadcast json, i can create a data structure:

```
DeploymentData:
  - blockNumber;
  - contractAddress;
  - contractName;
```

## Forth step: generate the `deployment.ts` file

with the above data, i want to populate a `.ts` array for my docs:

this:

```typescript
export const chains: Sablier.Deployment[] = [];
```

should become:

```typescript
export const chains: Sablier.Deployment[] = [
  get(chains.<chain_name>.id, {
    [manifest.<contract_name_upper_snake_cases>]: ["<contract_address>", <block_number_deci_format>],
    [manifest.<contract_name_upper_snake_cases>]: ["<contract_address>", <block_number_deci_format>],
    [manifest.<contract_name_upper_snake_cases>]: ["<contract_address>", <block_number_deci_format>],
    [manifest.<contract_name_upper_snake_cases>]: ["<contract_address>", <block_number_deci_format>],
  }),
];
```

the above example, is for the cases when there are 4 contracts deployed, during the forge script execution, i.e. found in broadcast json.

this `get` should be per each chain deployed
