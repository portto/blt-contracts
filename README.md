# Blocto Token Contracts

## Setup Flow CLI
https://docs.onflow.org/flow-cli/install

## Run Scripts/Transactions - Examples
### Setup Blocto Token Vault
```
flow transactions send ./transactions/token/setupBloctoTokenVault.cdc \
  --network testnet \
  --signer blt-user-testnet \
  --gas-limit 1000
```

### Transfer Blocto Token
```
flow transactions send ./transactions/token/transferBloctoToken.cdc \
  --network testnet \
  --arg UFix64:100.0 \
  --arg Address:0x03d1e02a48354e2b \
  --signer blt-admin-testnet \
  --gas-limit 1000
```

### Setup BloctoPass Collection
```
flow transactions send ./transactions/token/setupBloctoPassCollection.cdc \
  --network testnet \
  --signer blt-user-testnet \
  --gas-limit 1000
```

### Mint BloctoPass NFT
```
flow transactions send ./transactions/token/mintBloctoPass.cdc \
  --network testnet \
  --signer blt-admin-testnet \
  --gas-limit 1000
```

### Get Blocto Token Balance
```
flow scripts execute ./scripts/token/getBloctoTokenBalance.cdc \
  --network testnet \
  --arg Address:0x03d1e02a48354e2b
```

### Stake BLT into BloctoPass
```
flow transactions send ./transactions/staking/stakeNewTokens.cdc \
  --network testnet \
  --arg UFix64:1000.0 \
  --signer blt-user-testnet \
  --gas-limit 1000
```

### Get Staking Info
```
flow scripts execute ./scripts/staking/getStakingInfo.cdc \
  --network testnet \
  --arg Address:0x03d1e02a48354e2b
```

### Switch Epoch
```
flow transactions send ./transactions/staking/switchEpoch.cdc \
  --network testnet \
  --signer blt-admin-testnet \
  --gas-limit 1000
```
