// File: lib.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Blockchain integration for AURIA Runtime Core.
//     Provides Ethereum JSON-RPC client, wallet management, contract interactions
//     for Settlement, LicenseRegistry, and ShardRegistry Solidity contracts.

pub mod client;
pub mod wallet;
pub mod contracts;
pub mod transaction;
pub mod events;

pub use client::*;
pub use wallet::*;
pub use contracts::*;
pub use transaction::*;
pub use events::*;
