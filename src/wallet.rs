// File: wallet.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Wallet and key management for blockchain interactions.
//     Provides address derivation and transaction signing utilities.

use auria_core::{AuriaError, AuriaResult};
use rand::Rng;
use sha3::{Digest, Keccak256};

#[derive(Clone)]
pub struct Wallet {
    private_key: [u8; 32],
}

impl Wallet {
    pub fn new() -> AuriaResult<Self> {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        Ok(Self { private_key: key })
    }

    pub fn from_secret_key(secret_key_bytes: [u8; 32]) -> Self {
        Self {
            private_key: secret_key_bytes,
        }
    }

    pub fn from_mnemonic(mnemonic: &str) -> AuriaResult<Self> {
        let seed = Self::mnemonic_to_seed(mnemonic)?;
        let secret_key_bytes: [u8; 32] = seed[..32]
            .try_into()
            .map_err(|_| AuriaError::ExecutionError("Invalid seed".to_string()))?;
        Ok(Self {
            private_key: secret_key_bytes,
        })
    }

    fn mnemonic_to_seed(mnemonic: &str) -> AuriaResult<Vec<u8>> {
        let salt = "auria-wallet";
        let mut hasher = Keccak256::new();
        hasher.update(mnemonic.as_bytes());
        hasher.update(salt.as_bytes());
        let mut result = hasher.finalize().to_vec();

        for _ in 0..10000 {
            let mut hasher = Keccak256::new();
            hasher.update(&result);
            result = hasher.finalize().to_vec();
        }

        Ok(result)
    }

    pub fn address(&self) -> String {
        let public_key = self.public_key();
        let hash = Keccak256::digest(&public_key);
        let address_bytes = &hash[12..];
        format!("0x{}", hex::encode(address_bytes))
    }

    pub fn public_key(&self) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(&self.private_key);
        hasher.finalize().to_vec()
    }

    pub fn sign_message(&self, message: &[u8]) -> Signature {
        let mut hasher = Keccak256::new();
        hasher.update(&self.private_key);
        hasher.update(message);
        let hash = hasher.finalize();

        Signature {
            r: hash[..32].to_vec(),
            s: hash[..32].to_vec(),
            v: 27,
        }
    }

    pub fn sign_transaction(&self, tx: &TransactionSignData) -> Vec<u8> {
        let encoded = tx.encode();
        let mut hasher = Keccak256::new();
        hasher.update(&self.private_key);
        hasher.update(&encoded);
        let hash = hasher.finalize();

        let mut result = encoded;

        let chain_id = tx.chain_id;
        let v = if chain_id > 0 { 35 + chain_id * 2 } else { 27 };
        result.push(v as u8);

        result.extend_from_slice(&hash[..32]);
        result.extend_from_slice(&hash[..32]);

        result
    }

    pub fn secret_key_bytes(&self) -> [u8; 32] {
        self.private_key
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new().expect("Failed to generate wallet")
    }
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub r: Vec<u8>,
    pub s: Vec<u8>,
    pub v: u8,
}

impl Signature {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(65);
        result.extend_from_slice(&self.r);
        result.extend_from_slice(&self.s);
        result.push(self.v);
        result
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }
}

#[derive(Clone, Debug)]
pub struct TransactionSignData {
    pub nonce: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub to: Vec<u8>,
    pub value: u64,
    pub data: Vec<u8>,
    pub chain_id: u64,
}

impl TransactionSignData {
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&encode_rlp_scalar(self.nonce));
        result.extend_from_slice(&encode_rlp_scalar(self.gas_price));
        result.extend_from_slice(&encode_rlp_scalar(self.gas_limit));
        result.extend_from_slice(&encode_rlp_string(&self.to));
        result.extend_from_slice(&encode_rlp_scalar(self.value));
        result.extend_from_slice(&encode_rlp_string(&self.data));

        if self.chain_id > 0 {
            result.extend_from_slice(&encode_rlp_scalar(self.chain_id));
            result.extend_from_slice(&encode_rlp_scalar(0u64));
            result.extend_from_slice(&encode_rlp_scalar(0u64));
        }

        let len = result.len();
        if len < 56 {
            let mut prefix = vec![192 + len as u8];
            prefix.append(&mut result);
            prefix
        } else {
            let len_bytes = len.to_be_bytes();
            let mut prefix = vec![247 + len_bytes.len() as u8];
            prefix.extend_from_slice(&len_bytes);
            prefix.extend_from_slice(&result);
            prefix
        }
    }
}

fn encode_rlp_scalar(n: u64) -> Vec<u8> {
    if n == 0 {
        vec![0x80]
    } else {
        let bytes = n.to_be_bytes();
        let leading_zeros = bytes.iter().take_while(|&&b| b == 0).count();
        let trimmed = &bytes[leading_zeros..];
        if trimmed.len() == 1 && trimmed[0] < 128 {
            trimmed.to_vec()
        } else {
            let mut result = vec![128 + trimmed.len() as u8];
            result.extend_from_slice(trimmed);
            result
        }
    }
}

fn encode_rlp_string(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        vec![0x80]
    } else if data.len() == 1 && data[0] < 128 {
        data.to_vec()
    } else if data.len() < 56 {
        let mut result = vec![128 + data.len() as u8];
        result.extend_from_slice(data);
        result
    } else {
        let len_bytes = data.len().to_be_bytes();
        let leading_zeros = len_bytes.iter().take_while(|&&b| b == 0).count();
        let trimmed = &len_bytes[leading_zeros..];
        let mut result = vec![183 + trimmed.len() as u8];
        result.extend_from_slice(trimmed);
        result.extend_from_slice(data);
        result
    }
}

pub fn validate_address(address: &str) -> bool {
    if !address.starts_with("0x") {
        return false;
    }
    if address.len() != 42 {
        return false;
    }
    hex::decode(&address[2..]).is_ok()
}

pub fn parse_address(address: &str) -> AuriaResult<Vec<u8>> {
    if !validate_address(address) {
        return Err(AuriaError::ExecutionError(
            "Invalid address format".to_string(),
        ));
    }
    hex::decode(&address[2..])
        .map_err(|e| AuriaError::ExecutionError(format!("Failed to parse address: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_generation() {
        let wallet = Wallet::new().unwrap();
        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }

    #[test]
    fn test_sign_transaction() {
        let wallet = Wallet::new().unwrap();

        let tx = TransactionSignData {
            nonce: 0,
            gas_price: 20_000_000_000,
            gas_limit: 21000,
            to: parse_address("0x1234567890123456789012345678901234567890").unwrap(),
            value: 1000000000000000000,
            data: vec![],
            chain_id: 1,
        };

        let signed = wallet.sign_transaction(&tx);
        assert!(signed.len() > 0);
    }

    #[test]
    fn test_address_validation() {
        assert!(validate_address(
            "0x1234567890123456789012345678901234567890"
        ));
        assert!(!validate_address("0x123"));
        assert!(!validate_address(
            "1234567890123456789012345678901234567890"
        ));
    }

    #[test]
    fn test_from_mnemonic() {
        let wallet = Wallet::from_mnemonic("test mnemonic phrase for auria wallet").unwrap();
        let address = wallet.address();
        assert!(address.starts_with("0x"));
    }
}
