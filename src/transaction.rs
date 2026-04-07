// File: transaction.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Transaction building and utilities for blockchain interactions.
//     Provides EIP-1559 transaction support, gas estimation, and transaction management.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub value: Option<u64>,
    pub data: Option<String>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
    pub max_fee_per_gas: Option<u64>,
    pub max_priority_fee_per_gas: Option<u64>,
    pub nonce: Option<u64>,
    pub chain_id: u64,
}

impl Transaction {
    pub fn new(from: String, to: String, chain_id: u64) -> Self {
        Self {
            from,
            to,
            value: None,
            data: None,
            gas_limit: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            chain_id,
        }
    }

    pub fn with_value(mut self, value: u64) -> Self {
        self.value = Some(value);
        self
    }

    pub fn with_data(mut self, data: String) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    pub fn with_gas_price(mut self, price: u64) -> Self {
        self.gas_price = Some(price);
        self
    }

    pub fn with_eip1559_fees(mut self, max_fee: u64, priority_fee: u64) -> Self {
        self.max_fee_per_gas = Some(max_fee);
        self.max_priority_fee_per_gas = Some(priority_fee);
        self
    }

    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn to_transaction_request(&self) -> crate::client::TransactionRequest {
        let gas_price = self.gas_price.or(self.max_fee_per_gas);

        crate::client::TransactionRequest {
            from: Some(self.from.clone()),
            to: self.to.clone(),
            gas: self.gas_limit.map(|g| format!("0x{:x}", g)),
            gas_price: gas_price.map(|p| format!("0x{:x}", p)),
            value: self.value.map(|v| format!("0x{:x}", v)),
            data: self.data.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub hash: String,
    pub block_number: Option<u64>,
    pub confirmed: bool,
    pub success: bool,
    pub gas_used: Option<u64>,
    pub logs: Vec<TransactionLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}

pub struct TransactionManager {
    chain_id: u64,
    nonce: u64,
    gas_price: u64,
    gas_limit: u64,
}

impl TransactionManager {
    pub fn new(chain_id: u64) -> Self {
        Self {
            chain_id,
            nonce: 0,
            gas_price: 20_000_000_000,
            gas_limit: 100_000,
        }
    }

    pub fn with_gas_price(mut self, price: u64) -> Self {
        self.gas_price = price;
        self
    }

    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    pub fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }

    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    pub fn get_next_nonce(&self) -> u64 {
        self.nonce
    }

    pub fn get_gas_price(&self) -> u64 {
        self.gas_price
    }

    pub fn get_chain_id(&self) -> u64 {
        self.chain_id
    }
}

pub fn estimate_total_cost(gas_limit: u64, gas_price: u64) -> u64 {
    gas_limit.saturating_mul(gas_price)
}

pub fn calculate_eip1559_fees(base_fee: u64, max_priority_fee: u64) -> (u64, u64) {
    let max_fee = base_fee.saturating_mul(2).saturating_add(max_priority_fee);
    (max_fee, max_priority_fee)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "0x1234567890123456789012345678901234567890".to_string(),
            "0x0987654321098765432109876543210987654321".to_string(),
            1,
        );

        assert_eq!(tx.chain_id, 1);
        assert_eq!(tx.from, "0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_transaction_with_values() {
        let tx = Transaction::new(
            "0x1234567890123456789012345678901234567890".to_string(),
            "0x0987654321098765432109876543210987654321".to_string(),
            1,
        )
        .with_value(1000000000000000000)
        .with_gas_limit(21000)
        .with_nonce(5);

        assert_eq!(tx.value, Some(1000000000000000000));
        assert_eq!(tx.gas_limit, Some(21000));
        assert_eq!(tx.nonce, Some(5));
    }

    #[test]
    fn test_cost_estimation() {
        let cost = estimate_total_cost(21000, 50_000_000_000);
        assert_eq!(cost, 1_050_000_000_000_000);
    }

    #[test]
    fn test_eip1559_fees() {
        let (max_fee, priority_fee) = calculate_eip1559_fees(30_000_000_000, 2_000_000_000);
        assert_eq!(max_fee, 62_000_000_000);
        assert_eq!(priority_fee, 2_000_000_000);
    }
}
