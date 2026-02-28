// File: contracts.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Solidity contract bindings for AURIA blockchain contracts.
//     Provides interaction with Settlement, LicenseRegistry, and ShardRegistry.

use crate::client::{EthereumClient, TransactionRequest};
use crate::wallet::Wallet;
use auria_core::{AuriaError, AuriaResult};
use serde::{Deserialize, Serialize};

pub struct Contract {
    pub address: String,
    pub abi: ContractABI,
    client: EthereumClient,
    wallet: Wallet,
}

#[derive(Clone)]
pub struct ContractABI {
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Param>,
    pub outputs: Vec<Param>,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettlementReceipt {
    pub receipt_id: String,
    pub event_ids: Vec<String>,
    pub node_identity: String,
    pub timestamp: u64,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShardRegistration {
    pub shard_id: String,
    pub owner: String,
    pub metadata: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_id: String,
    pub owner: String,
    pub model_id: String,
    pub price_per_token: u64,
    pub active: bool,
}

impl Contract {
    pub fn new(address: String, client: EthereumClient, wallet: Wallet) -> Self {
        Self {
            address,
            abi: ContractABI::default(),
            client,
            wallet,
        }
    }

    pub fn encode_function_call(&self, function_name: &str, params: &[String]) -> AuriaResult<String> {
        let selector = compute_function_selector(function_name);
        let encoded_params = encode_params(params)?;
        
        let mut result = selector;
        result.extend_from_slice(&encoded_params);
        
        Ok(format!("0x{}", hex::encode(result)))
    }

    pub async fn call(&self, function_name: &str, params: Vec<String>) -> AuriaResult<String> {
        let data = self.encode_function_call(function_name, &params)?;
        
        let request = TransactionRequest {
            from: Some(self.wallet.address()),
            to: self.address.clone(),
            gas: None,
            gas_price: None,
            value: None,
            data: Some(data),
        };
        
        self.client.eth_call(request, None).await
    }

    pub async fn submit_transaction(
        &self,
        function_name: &str,
        params: Vec<String>,
        gas_limit: Option<u64>,
    ) -> AuriaResult<String> {
        let data = self.encode_function_call(function_name, &params)?;
        
        let from_address = self.wallet.address();
        let nonce = self.client.eth_get_transaction_count(&from_address).await?;
        let gas_price = self.client.eth_gas_price().await?;
        
        let request = TransactionRequest {
            from: Some(from_address.clone()),
            to: self.address.clone(),
            gas: None,
            gas_price: Some(format!("0x{:x}", gas_price)),
            value: None,
            data: Some(data.clone()),
        };
        
        let gas = match gas_limit {
            Some(g) => g,
            None => self.client.eth_estimate_gas(request).await.unwrap_or(100000),
        };
        
        let tx_data = crate::wallet::TransactionSignData {
            nonce,
            gas_price,
            gas_limit: gas,
            to: hex::decode(&self.address[2..]).unwrap_or_default(),
            value: 0,
            data: hex::decode(&data[2..]).unwrap_or_default(),
            chain_id: self.client.chain_id().await.unwrap_or(1),
        };
        
        let signed_tx = self.wallet.sign_transaction(&tx_data);
        
        self.client.eth_send_raw_transaction(&format!("0x{}", hex::encode(&signed_tx))).await
    }
}

pub struct SettlementContract {
    contract: Contract,
}

impl SettlementContract {
    pub fn new(client: EthereumClient, wallet: Wallet, address: String) -> Self {
        Self {
            contract: Contract::new(address, client, wallet),
        }
    }

    pub async fn submit_receipt(&self, receipt: &SettlementReceipt) -> AuriaResult<String> {
        let event_ids_json = serde_json::to_string(&receipt.event_ids)
            .map_err(|e| AuriaError::ExecutionError(format!("JSON error: {}", e)))?;
        
        let params = vec![
            receipt.receipt_id.clone(),
            event_ids_json,
            receipt.node_identity.clone(),
            format!("0x{:x}", receipt.timestamp),
            receipt.signature.clone(),
        ];
        
        self.contract.submit_transaction("submitReceipt", params, Some(200000)).await
    }

    pub async fn settle(&self, root: &str) -> AuriaResult<String> {
        self.contract.submit_transaction("settle", vec![root.to_string()], Some(100000)).await
    }

    pub async fn record_usage(&self, node: &str, amount: u64) -> AuriaResult<String> {
        let params = vec![
            node.to_string(),
            format!("0x{:x}", amount),
        ];
        
        self.contract.submit_transaction("recordUsage", params, Some(50000)).await
    }

    pub async fn get_reward(&self, node: &str) -> AuriaResult<String> {
        self.contract.call("getReward", vec![node.to_string()]).await
    }

    pub async fn withdraw(&self) -> AuriaResult<String> {
        self.contract.submit_transaction("withdraw", vec![], Some(50000)).await
    }

    pub async fn compute_merkle_root(&self, event_ids: Vec<String>) -> AuriaResult<String> {
        self.contract.call("computeMerkleRoot", event_ids).await
    }
}

pub struct LicenseRegistryContract {
    contract: Contract,
}

impl LicenseRegistryContract {
    pub fn new(client: EthereumClient, wallet: Wallet, address: String) -> Self {
        Self {
            contract: Contract::new(address, client, wallet),
        }
    }

    pub async fn register_license(
        &self,
        model_id: &str,
        price_per_token: u64,
        metadata: &str,
    ) -> AuriaResult<String> {
        let params = vec![
            model_id.to_string(),
            format!("0x{:x}", price_per_token),
            metadata.to_string(),
        ];
        
        self.contract.submit_transaction("registerLicense", params, Some(150000)).await
    }

    pub async fn update_price(&self, license_id: &str, new_price: u64) -> AuriaResult<String> {
        let params = vec![
            license_id.to_string(),
            format!("0x{:x}", new_price),
        ];
        
        self.contract.submit_transaction("updatePrice", params, Some(50000)).await
    }

    pub async fn purchase_license(&self, license_id: &str, tokens: u64) -> AuriaResult<String> {
        let params = vec![
            license_id.to_string(),
            format!("0x{:x}", tokens),
        ];
        
        self.contract.submit_transaction("purchaseLicense", params, Some(100000)).await
    }

    pub async fn get_license_info(&self, license_id: &str) -> AuriaResult<LicenseInfo> {
        let result = self.contract.call("getLicenseInfo", vec![license_id.to_string()]).await?;
        decode_license_info(&result)
    }

    pub async fn verify_license(&self, license_id: &str, user: &str) -> AuriaResult<bool> {
        let result = self.contract.call("verifyLicense", vec![license_id.to_string(), user.to_string()]).await?;
        Ok(result != "0x0000000000000000000000000000000000000000000000000000000000000000")
    }
}

pub struct ShardRegistryContract {
    contract: Contract,
}

impl ShardRegistryContract {
    pub fn new(client: EthereumClient, wallet: Wallet, address: String) -> Self {
        Self {
            contract: Contract::new(address, client, wallet),
        }
    }

    pub async fn register_shard(
        &self,
        shard_id: &str,
        model_id: &str,
        capacity: u64,
        metadata: &str,
    ) -> AuriaResult<String> {
        let params = vec![
            shard_id.to_string(),
            model_id.to_string(),
            format!("0x{:x}", capacity),
            metadata.to_string(),
        ];
        
        self.contract.submit_transaction("registerShard", params, Some(150000)).await
    }

    pub async fn update_shard_status(&self, shard_id: &str, active: bool) -> AuriaResult<String> {
        let params = vec![
            shard_id.to_string(),
            if active { "true".to_string() } else { "false".to_string() },
        ];
        
        self.contract.submit_transaction("updateShardStatus", params, Some(50000)).await
    }

    pub async fn get_shard_info(&self, shard_id: &str) -> AuriaResult<ShardRegistration> {
        let result = self.contract.call("getShardInfo", vec![shard_id.to_string()]).await?;
        decode_shard_registration(&result)
    }

    pub async fn get_shard_owner(&self, shard_id: &str) -> AuriaResult<String> {
        self.contract.call("getShardOwner", vec![shard_id.to_string()]).await
    }

    pub async fn list_active_shards(&self, model_id: &str) -> AuriaResult<Vec<String>> {
        self.contract.call("listActiveShards", vec![model_id.to_string()]).await
        .map(|s| vec![s])
    }
}

fn compute_function_selector(function_name: &str) -> Vec<u8> {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(function_name.as_bytes());
    let hash = hasher.finalize();
    hash[..4].to_vec()
}

fn encode_params(params: &[String]) -> AuriaResult<Vec<u8>> {
    let mut result = Vec::new();
    for param in params {
        let bytes = if param.starts_with("0x") {
            hex::decode(&param[2..]).unwrap_or_else(|_| param.as_bytes().to_vec())
        } else {
            param.as_bytes().to_vec()
        };
        
        let padded = pad_to_32_bytes(&bytes);
        result.extend_from_slice(&padded);
    }
    Ok(result)
}

fn pad_to_32_bytes(data: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; 32usize];
    let offset = 32usize.saturating_sub(data.len());
    result[offset..].copy_from_slice(data);
    result
}

fn decode_license_info(data: &str) -> AuriaResult<LicenseInfo> {
    let bytes = hex::decode(data.trim_start_matches("0x"))
        .map_err(|e| AuriaError::ExecutionError(format!("Decode error: {}", e)))?;
    
    Ok(LicenseInfo {
        license_id: format!("0x{}", &hex::encode(&bytes[0..32])),
        owner: format!("0x{}", &hex::encode(&bytes[32..72])),
        model_id: format!("0x{}", &hex::encode(&bytes[72..104])),
        price_per_token: u64::from_be_bytes(bytes[104..112].try_into().unwrap_or_default()),
        active: bytes[112] != 0,
    })
}

fn decode_shard_registration(data: &str) -> AuriaResult<ShardRegistration> {
    let bytes = hex::decode(data.trim_start_matches("0x"))
        .map_err(|e| AuriaError::ExecutionError(format!("Decode error: {}", e)))?;
    
    Ok(ShardRegistration {
        shard_id: format!("0x{}", &hex::encode(&bytes[0..32])),
        owner: format!("0x{}", &hex::encode(&bytes[32..72])),
        metadata: String::from_utf8_lossy(&bytes[72..104]).to_string(),
        active: bytes[104] != 0,
    })
}

impl Default for ContractABI {
    fn default() -> Self {
        Self {
            functions: vec![
                Function {
                    name: "submitReceipt".to_string(),
                    inputs: vec![
                        Param { name: "receiptId".to_string(), param_type: "bytes32".to_string() },
                        Param { name: "eventIds".to_string(), param_type: "bytes32[]".to_string() },
                        Param { name: "nodeIdentity".to_string(), param_type: "address".to_string() },
                        Param { name: "timestamp".to_string(), param_type: "uint256".to_string() },
                        Param { name: "signature".to_string(), param_type: "bytes".to_string() },
                    ],
                    outputs: vec![],
                },
                Function {
                    name: "settle".to_string(),
                    inputs: vec![
                        Param { name: "root".to_string(), param_type: "bytes32".to_string() },
                    ],
                    outputs: vec![],
                },
                Function {
                    name: "recordUsage".to_string(),
                    inputs: vec![
                        Param { name: "node".to_string(), param_type: "address".to_string() },
                        Param { name: "amount".to_string(), param_type: "uint256".to_string() },
                    ],
                    outputs: vec![],
                },
            ],
        }
    }
}
