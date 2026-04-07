// File: client.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Ethereum JSON-RPC client for interacting with blockchain nodes.
//     Supports eth_sendRawTransaction, eth_call, eth_getTransactionReceipt,
//     eth_getBalance, eth_blockNumber, and other standard JSON-RPC methods.

use auria_core::{AuriaError, AuriaResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

#[derive(Clone)]
pub struct EthereumClient {
    client: Client,
    url: String,
    chain_id: Arc<RwLock<Option<u64>>>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: Option<String>,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionResponse {
    pub hash: String,
    pub nonce: String,
    pub block_hash: Option<String>,
    pub block_number: Option<String>,
    pub transaction_index: Option<String>,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas_price: String,
    pub gas: String,
    pub input: String,
    pub v: String,
    pub r: String,
    pub s: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub block_hash: Option<String>,
    pub block_number: Option<String>,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub contract_address: Option<String>,
    pub logs: Vec<Log>,
    pub logs_bloom: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: Option<String>,
    pub transaction_hash: Option<String>,
    pub log_index: Option<String>,
}

impl EthereumClient {
    pub fn new(url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            url,
            chain_id: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn chain_id(&self) -> AuriaResult<u64> {
        {
            let cached = self.chain_id.read().await;
            if let Some(id) = *cached {
                return Ok(id);
            }
        }

        let id = self.eth_chain_id().await?;
        {
            let mut cached = self.chain_id.write().await;
            *cached = Some(id);
        }
        Ok(id)
    }

    pub async fn eth_chain_id(&self) -> AuriaResult<u64> {
        let response: JsonRpcResponse<String> = self
            .request("eth_chainId", json!([]))
            .await?;
        
        let hex_str = response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to get chain id".to_string())
        })?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse chain id: {}", e)))
    }

    pub async fn eth_block_number(&self) -> AuriaResult<u64> {
        let response: JsonRpcResponse<String> = self
            .request("eth_blockNumber", json!([]))
            .await?;
        
        let hex_str = response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to get block number".to_string())
        })?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse block number: {}", e)))
    }

    pub async fn eth_get_balance(&self, address: &str) -> AuriaResult<u64> {
        let params = json!([address, "latest"]);
        let response: JsonRpcResponse<String> = self
            .request("eth_getBalance", params)
            .await?;
        
        let hex_str = response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to get balance".to_string())
        })?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse balance: {}", e)))
    }

    pub async fn eth_get_transaction_count(&self, address: &str) -> AuriaResult<u64> {
        let params = json!([address, "latest"]);
        let response: JsonRpcResponse<String> = self
            .request("eth_getTransactionCount", params)
            .await?;
        
        let hex_str = response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to get transaction count".to_string())
        })?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse nonce: {}", e)))
    }

    pub async fn eth_get_transaction_by_hash(&self, hash: &str) -> AuriaResult<Option<TransactionResponse>> {
        let params = json!([hash]);
        let response: JsonRpcResponse<TransactionResponse> = self
            .request("eth_getTransactionByHash", params)
            .await?;
        
        Ok(response.result)
    }

    pub async fn eth_get_transaction_receipt(&self, hash: &str) -> AuriaResult<Option<TransactionReceipt>> {
        let params = json!([hash]);
        let response: JsonRpcResponse<TransactionReceipt> = self
            .request("eth_getTransactionReceipt", params)
            .await?;
        
        Ok(response.result)
    }

    pub async fn eth_call(&self, request: TransactionRequest, block: Option<String>) -> AuriaResult<String> {
        let params = json!([request, block.unwrap_or_else(|| "latest".to_string())]);
        let response: JsonRpcResponse<String> = self
            .request("eth_call", params)
            .await?;
        
        response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to call contract".to_string())
        })
    }

    pub async fn eth_send_raw_transaction(&self, signed_tx: &str) -> AuriaResult<String> {
        let params = json!([signed_tx]);
        let response: JsonRpcResponse<String> = self
            .request("eth_sendRawTransaction", params)
            .await?;
        
        response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to send transaction".to_string())
        })
    }

    pub async fn eth_estimate_gas(&self, request: TransactionRequest) -> AuriaResult<u64> {
        let params = json!([request]);
        let response: JsonRpcResponse<String> = self
            .request("eth_estimateGas", params)
            .await?;
        
        let hex_str = response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to estimate gas".to_string())
        })?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse gas estimate: {}", e)))
    }

    pub async fn eth_gas_price(&self) -> AuriaResult<u64> {
        let response: JsonRpcResponse<String> = self
            .request("eth_gasPrice", json!([]))
            .await?;
        
        let hex_str = response.result.ok_or_else(|| {
            AuriaError::ExecutionError("Failed to get gas price".to_string())
        })?;
        
        u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
            .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse gas price: {}", e)))
    }

    pub async fn wait_for_transaction(&self, tx_hash: &str, timeout_secs: u64) -> AuriaResult<TransactionReceipt> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_secs(2);

        loop {
            if start.elapsed().as_secs() > timeout_secs {
                return Err(AuriaError::ExecutionError(
                    "Transaction receipt timeout".to_string()
                ));
            }

            if let Some(receipt) = self.eth_get_transaction_receipt(tx_hash).await? {
                if receipt.block_number.is_some() {
                    return Ok(receipt);
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(&self, method: &str, params: serde_json::Value) -> AuriaResult<T> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        let response = self.client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AuriaError::ExecutionError(format!("HTTP request failed: {}", e)))?;

        let rpc_response: JsonRpcResponse<T> = response
            .json()
            .await
            .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AuriaError::ExecutionError(format!(
                "JSON-RPC error {}: {}", error.code, error.message
            )));
        }

        rpc_response.result.ok_or_else(|| {
            AuriaError::ExecutionError("No result in response".to_string())
        })
    }
}

impl Default for EthereumClient {
    fn default() -> Self {
        Self::new("http://localhost:8545".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = EthereumClient::new("http://localhost:8545".to_string());
        assert_eq!(client.url, "http://localhost:8545");
    }
}
