# auria-blockchain

Blockchain integration for AURIA Runtime Core.

## Overview

`auria-blockchain` provides Ethereum blockchain integration for the AURIA decentralized LLM. It enables interaction with smart contracts for settlement, licensing, and shard registration.

## Features

- **Ethereum JSON-RPC Client** - Full support for Ethereum node communication
- **Wallet Management** - Key generation, signing, and address derivation
- **Smart Contract Bindings** - Pre-built integration with:
  - `Settlement` - Usage tracking and payment settlement
  - `LicenseRegistry` - License management for AI models
  - `ShardRegistry` - Shard registration and ownership
- **Transaction Management** - EIP-1559 support, gas estimation, nonce management
- **Event Parsing** - Decode and watch Solidity events

## Installation

```toml
[dependencies]
auria-blockchain = "0.1.0"
```

## Quick Start

```rust
use auria_blockchain::{EthereumClient, Wallet, SettlementContract};

// Create a wallet
let wallet = Wallet::new()?;

// Connect to Ethereum node
let client = EthereumClient::new("https://mainnet.infura.io/v3/YOUR_PROJECT_ID");

// Create settlement contract
let settlement = SettlementContract::new(
    client,
    wallet,
    "0x...".to_string() // contract address
);
```

## Modules

### client

Ethereum JSON-RPC client with support for:
- `eth_chainId` - Get chain ID
- `eth_blockNumber` - Get current block
- `eth_getBalance` - Get account balance
- `eth_sendRawTransaction` - Submit signed transactions
- `eth_call` - Contract read calls
- `eth_estimateGas` - Gas estimation

### wallet

Wallet and key management:
- Generate new wallets from random entropy
- Import from secret key or mnemonic
- Address derivation (Keccak256)
- Transaction signing (EIP-155)

### contracts

Solidity contract bindings:

```rust
// Settlement contract
settlement.submit_receipt(&receipt).await?;
settlement.settle(&root).await?;
settlement.record_usage(node, amount).await?;
settlement.withdraw().await?;

// License registry
license_registry.register_license(model_id, price, metadata).await?;
license_registry.purchase_license(license_id, tokens).await?;

// Shard registry
shard_registry.register_shard(shard_id, model_id, capacity, metadata).await?;
shard_registry.update_shard_status(shard_id, active).await?;
```

### transaction

Transaction building utilities:
- EIP-1559 fee market support
- Gas price estimation
- Nonce management
- Transaction status tracking

### events

Event parsing for Solidity contracts:
- Event signature computation
- Log decoding
- Standard AURIA events (ReceiptSubmitted, SettlementCompleted, RewardDistributed)

## Example: Submit Usage Receipt

```rust
use auria_blockchain::{EthereumClient, Wallet, SettlementContract, SettlementReceipt};

async fn submit_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::new()?;
    let client = EthereumClient::new("https://sepolia.infura.io/v3/YOUR_KEY");
    
    let settlement = SettlementContract::new(
        client,
        wallet,
        "0xContractAddress".to_string()
    );
    
    let receipt = SettlementReceipt {
        receipt_id: "0x...".to_string(),
        event_ids: vec!["0x...".to_string()],
        node_identity: wallet.address(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        signature: "0x...".to_string(),
    };
    
    let tx_hash = settlement.submit_receipt(&receipt).await?;
    println!("Transaction submitted: {}", tx_hash);
    
    Ok(())
}
```

## Example: Create Wallet from Mnemonic

```rust
use auria_blockchain::Wallet;

let wallet = Wallet::from_mnemonic(
    "your twelve word mnemonic phrase here"
)?;

println!("Address: {}", wallet.address());
println!("Public Key: {}", wallet.public_key());
```

## Gas Management

```rust
use auria_blockchain::{Transaction, TransactionManager};

// Manual gas estimation
let tx = Transaction::new(from, to, chain_id)
    .with_value(1000000000000000000) // 1 ETH
    .with_gas_limit(21000)
    .with_gas_price(50000000000); // 50 Gwei

// Or use the manager for automatic nonce/gas management
let mut manager = TransactionManager::new(chain_id);
manager.set_nonce(nonce);
manager.with_gas_price(50000000000);

let cost = estimate_total_cost(gas_limit, gas_price);
```

## Testing

```bash
cargo test --package auria-blockchain
```

## Dependencies

- `auria-core` - Core types and error handling
- `auria-settlement` - Settlement logic
- `tokio` - Async runtime
- `reqwest` - HTTP client for JSON-RPC
- `sha3` - Keccak256 hashing
- `serde` - Serialization

## License

Apache-2.0
