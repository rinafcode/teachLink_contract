use soroban_sdk::contractevent;

use crate::types::{BridgeTransaction, CrossChainMessage};
use soroban_sdk::{Address, Bytes};

#[contractevent]
#[derive(Clone, Debug)]
pub struct DepositEvent {
    pub nonce: u64,
    pub from: Address,
    pub amount: i128,
    pub destination_chain: u32,
    pub destination_address: Bytes,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct ReleaseEvent {
    pub nonce: u64,
    pub recipient: Address,
    pub amount: i128,
    pub source_chain: u32,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeInitiatedEvent {
    pub nonce: u64,
    pub transaction: BridgeTransaction,
}

#[contractevent]
#[derive(Clone, Debug)]
pub struct BridgeCompletedEvent {
    pub nonce: u64,
    pub message: CrossChainMessage,
}
