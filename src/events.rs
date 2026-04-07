// File: events.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Event parsing and subscription utilities for blockchain events.
//     Handles Solidity event decoding, log parsing, and event watching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: Option<u64>,
    pub transaction_hash: Option<String>,
    pub log_index: Option<u64>,
}

impl Event {
    pub fn parse_event_signature(&self) -> Option<String> {
        self.topics.first().map(|t| {
            let bytes = hex::decode(t.trim_start_matches("0x")).ok();
            match bytes {
                Some(b) if b.len() >= 32 => {
                    let hash = &b[0..32];
                    format!("0x{}", hex::encode(hash))
                }
                _ => t.clone(),
            }
        })
    }

    pub fn decode_event(&self, event_abi: &EventABI) -> Option<DecodedEvent> {
        let signature = self.parse_event_signature()?;

        if signature != event_abi.signature {
            return None;
        }

        let mut values = HashMap::new();

        for (i, param) in event_abi.inputs.iter().enumerate() {
            let topic_index = i + 1;
            if topic_index < self.topics.len() {
                let topic = &self.topics[topic_index];
                values.insert(param.name.clone(), EventValue::Topic(topic.clone()));
            }
        }

        let data_bytes = hex::decode(self.data.trim_start_matches("0x")).ok();
        if let Some(bytes) = data_bytes {
            let mut offset = 0;
            for param in &event_abi.inputs {
                if offset < bytes.len() {
                    let value = decode_param(&bytes[offset..], &param.param_type);
                    values.entry(param.name.clone()).or_insert(value);
                    offset += 32;
                }
            }
        }

        Some(DecodedEvent {
            name: event_abi.name.clone(),
            values,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EventABI {
    pub name: String,
    pub signature: String,
    pub inputs: Vec<EventParam>,
}

#[derive(Debug, Clone)]
pub struct EventParam {
    pub name: String,
    pub param_type: String,
    pub indexed: bool,
}

#[derive(Debug, Clone)]
pub enum EventValue {
    Topic(String),
    Data(Vec<u8>),
    Unknown,
}

impl EventValue {
    pub fn as_string(&self) -> String {
        match self {
            EventValue::Topic(t) => t.clone(),
            EventValue::Data(d) => format!("0x{}", hex::encode(d)),
            EventValue::Unknown => String::new(),
        }
    }

    pub fn as_address(&self) -> Option<String> {
        match self {
            EventValue::Topic(t) => {
                let bytes = hex::decode(t.trim_start_matches("0x")).ok()?;
                if bytes.len() >= 32 {
                    Some(format!("0x{}", &hex::encode(&bytes[12..32])))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_uint(&self) -> Option<u64> {
        match self {
            EventValue::Topic(t) => {
                let bytes = hex::decode(t.trim_start_matches("0x")).ok()?;
                if bytes.len() >= 32 {
                    Some(u64::from_be_bytes(bytes[24..32].try_into().ok()?))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecodedEvent {
    pub name: String,
    pub values: HashMap<String, EventValue>,
}

impl DecodedEvent {
    pub fn get(&self, key: &str) -> Option<&EventValue> {
        self.values.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).map(|v| v.as_string())
    }

    pub fn get_address(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_address())
    }

    pub fn get_uint(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|v| v.as_uint())
    }
}

fn decode_param(bytes: &[u8], param_type: &str) -> EventValue {
    if bytes.len() < 32 {
        return EventValue::Unknown;
    }

    match param_type {
        "address" => {
            let addr = &bytes[12..32];
            EventValue::Data(addr.to_vec())
        }
        "uint256" | "uint128" | "uint64" | "uint32" | "uint8" => EventValue::Data(bytes.to_vec()),
        "bytes32" | "bytes" => EventValue::Data(bytes.to_vec()),
        "bool" => EventValue::Data(bytes[31..32].to_vec()),
        _ => EventValue::Data(bytes.to_vec()),
    }
}

pub fn compute_event_signature(name: &str) -> String {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(&hash[..32]))
}

pub fn get_standard_events() -> HashMap<String, EventABI> {
    let mut events = HashMap::new();

    events.insert(
        compute_event_signature("ReceiptSubmitted(bytes32 indexed receiptId, bytes32 root)"),
        EventABI {
            name: "ReceiptSubmitted".to_string(),
            signature: compute_event_signature(
                "ReceiptSubmitted(bytes32 indexed receiptId, bytes32 root)",
            ),
            inputs: vec![
                EventParam {
                    name: "receiptId".to_string(),
                    param_type: "bytes32".to_string(),
                    indexed: true,
                },
                EventParam {
                    name: "root".to_string(),
                    param_type: "bytes32".to_string(),
                    indexed: false,
                },
            ],
        },
    );

    events.insert(
        compute_event_signature("SettlementCompleted(bytes32 root, uint256 timestamp)"),
        EventABI {
            name: "SettlementCompleted".to_string(),
            signature: compute_event_signature(
                "SettlementCompleted(bytes32 root, uint256 timestamp)",
            ),
            inputs: vec![
                EventParam {
                    name: "root".to_string(),
                    param_type: "bytes32".to_string(),
                    indexed: false,
                },
                EventParam {
                    name: "timestamp".to_string(),
                    param_type: "uint256".to_string(),
                    indexed: false,
                },
            ],
        },
    );

    events.insert(
        compute_event_signature("RewardDistributed(address indexed node, uint256 amount)"),
        EventABI {
            name: "RewardDistributed".to_string(),
            signature: compute_event_signature(
                "RewardDistributed(address indexed node, uint256 amount)",
            ),
            inputs: vec![
                EventParam {
                    name: "node".to_string(),
                    param_type: "address".to_string(),
                    indexed: true,
                },
                EventParam {
                    name: "amount".to_string(),
                    param_type: "uint256".to_string(),
                    indexed: false,
                },
            ],
        },
    );

    events.insert(
        compute_event_signature("LicenseRegistered(bytes32 indexed licenseId, address indexed owner, bytes32 modelId)"),
        EventABI {
            name: "LicenseRegistered".to_string(),
            signature: compute_event_signature("LicenseRegistered(bytes32 indexed licenseId, address indexed owner, bytes32 modelId)"),
            inputs: vec![
                EventParam { name: "licenseId".to_string(), param_type: "bytes32".to_string(), indexed: true },
                EventParam { name: "owner".to_string(), param_type: "address".to_string(), indexed: true },
                EventParam { name: "modelId".to_string(), param_type: "bytes32".to_string(), indexed: false },
            ],
        },
    );

    events.insert(
        compute_event_signature(
            "ShardRegistered(bytes32 indexed shardId, address indexed owner, uint256 capacity)",
        ),
        EventABI {
            name: "ShardRegistered".to_string(),
            signature: compute_event_signature(
                "ShardRegistered(bytes32 indexed shardId, address indexed owner, uint256 capacity)",
            ),
            inputs: vec![
                EventParam {
                    name: "shardId".to_string(),
                    param_type: "bytes32".to_string(),
                    indexed: true,
                },
                EventParam {
                    name: "owner".to_string(),
                    param_type: "address".to_string(),
                    indexed: true,
                },
                EventParam {
                    name: "capacity".to_string(),
                    param_type: "uint256".to_string(),
                    indexed: false,
                },
            ],
        },
    );

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_signature_computation() {
        let sig = compute_event_signature("Transfer(address from, address to, uint256 amount)");
        assert!(sig.starts_with("0x"));
        assert_eq!(sig.len(), 66);
    }

    #[test]
    fn test_standard_events() {
        let events = get_standard_events();
        assert!(events.contains_key(&compute_event_signature(
            "ReceiptSubmitted(bytes32 indexed receiptId, bytes32 root)"
        )));
    }

    #[test]
    fn test_event_parsing() {
        let event = Event {
            address: "0x1234567890123456789012345678901234567890".to_string(),
            topics: vec![
                compute_event_signature("RewardDistributed(address indexed node, uint256 amount)"),
                "0x0000000000000000000000001234567890123456789012345678901234567890".to_string(),
            ],
            data: "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
            block_number: Some(12345),
            transaction_hash: Some("0xabcdef".to_string()),
            log_index: Some(0),
        };

        let events = get_standard_events();
        let sig = event.parse_event_signature().unwrap();
        let abi = events.get(&sig).cloned().unwrap();

        let decoded = event.decode_event(&abi);
        assert!(decoded.is_some());
    }
}
