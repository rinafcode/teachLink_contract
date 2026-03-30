//! Cross-Chain Integration Testing Framework
//! 
//! This module provides comprehensive integration testing for all cross-chain operations
//! including bridge transfers, atomic swaps, message passing, and multi-chain support.

pub mod bridge_integration;
pub mod atomic_swap_integration;
pub mod message_passing_integration;
pub mod multichain_integration;
pub mod mock_chains;
pub mod test_utils;
pub mod failure_scenarios;

pub use bridge_integration::*;
pub use atomic_swap_integration::*;
pub use message_passing_integration::*;
pub use multichain_integration::*;
pub use mock_chains::*;
pub use test_utils::*;
pub use failure_scenarios::*;
