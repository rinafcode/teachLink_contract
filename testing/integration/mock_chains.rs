//! Mock blockchain implementations for testing cross-chain operations

use soroban_sdk::{Bytes, Address, Env, Map, Vec};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Mock chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockChainConfig {
    pub chain_id: u32,
    pub name: String,
    pub block_time: u64,
    pub finality_blocks: u32,
    pub gas_price: u64,
    pub bridge_contract: String,
}

/// Mock chain response for cross-chain operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MockChainResponse {
    Success { 
        tx_hash: String,
        block_number: u64,
        gas_used: u64,
    },
    Failure { 
        error: String,
        revert_reason: String,
    },
    Pending { 
        tx_hash: String,
        confirmations: u32,
    },
    Timeout { 
        message: String,
    },
}

/// Mock blockchain implementation
pub struct MockChain {
    pub config: MockChainConfig,
    pub blocks: HashMap<u64, MockBlock>,
    pub pending_txs: HashMap<String, MockTransaction>,
    pub current_block: u64,
    pub failure_mode: Option<MockFailureMode>,
}

#[derive(Debug, Clone)]
pub struct MockBlock {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
    pub transactions: Vec<MockTransaction>,
}

#[derive(Debug, Clone)]
pub struct MockTransaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: String,
    pub gas_used: u64,
    pub status: MockTxStatus,
}

#[derive(Debug, Clone)]
pub enum MockTxStatus {
    Success,
    Failure(String),
    Pending,
}

#[derive(Debug, Clone)]
pub enum MockFailureMode {
    AlwaysFail,
    RandomFail(f64), // probability 0.0-1.0
    Timeout,
    Revert(String),
}

impl MockChain {
    pub fn new(chain_id: u32, name: &str) -> Self {
        let config = MockChainConfig {
            chain_id,
            name: name.to_string(),
            block_time: match name {
                "Stellar" => 5,
                "Ethereum" => 12,
                "Polygon" => 2,
                "BSC" => 3,
                _ => 10,
            },
            finality_blocks: match name {
                "Stellar" => 1,
                "Ethereum" => 12,
                "Polygon" => 20,
                "BSC" => 3,
                _ => 6,
            },
            gas_price: match name {
                "Stellar" => 100,
                "Ethereum" => 20000,
                "Polygon" => 30000,
                "BSC" => 5000,
                _ => 10000,
            },
            bridge_contract: format!("bridge_{}", chain_id),
        };

        let genesis_block = MockBlock {
            number: 0,
            hash: format!("genesis_hash_{}", chain_id),
            timestamp: 1640995200, // Jan 1, 2022
            transactions: Vec::new(),
        };

        let mut blocks = HashMap::new();
        blocks.insert(0, genesis_block);

        Self {
            config,
            blocks,
            pending_txs: HashMap::new(),
            current_block: 0,
            failure_mode: None,
        }
    }

    pub fn set_failure_mode(&mut self, mode: MockFailureMode) {
        self.failure_mode = Some(mode);
    }

    pub fn clear_failure_mode(&mut self) {
        self.failure_mode = None;
    }

    pub fn advance_blocks(&mut self, count: u64) {
        for _ in 0..count {
            self.mine_block();
        }
    }

    pub fn mine_block(&mut self) {
        self.current_block += 1;
        let block = MockBlock {
            number: self.current_block,
            hash: format!("block_hash_{}_{}", self.config.chain_id, self.current_block),
            timestamp: 1640995200 + (self.current_block * self.config.block_time),
            transactions: Vec::new(),
        };
        self.blocks.insert(self.current_block, block);
    }

    pub fn submit_transaction(&mut self, tx: MockTransaction) -> String {
        let tx_hash = tx.hash.clone();
        
        // Check failure mode
        if let Some(failure_mode) = &self.failure_mode {
            match failure_mode {
                MockFailureMode::AlwaysFail => {
                    let failed_tx = MockTransaction {
                        status: MockTxStatus::Failure("Mock failure".to_string()),
                        ..tx
                    };
                    self.pending_txs.insert(tx_hash.clone(), failed_tx);
                    return tx_hash;
                }
                MockFailureMode::RandomFail(prob) => {
                    if rand::random::<f64>() < *prob {
                        let failed_tx = MockTransaction {
                            status: MockTxStatus::Failure("Random failure".to_string()),
                            ..tx
                        };
                        self.pending_txs.insert(tx_hash.clone(), failed_tx);
                        return tx_hash;
                    }
                }
                MockFailureMode::Timeout => {
                    let pending_tx = MockTransaction {
                        status: MockTxStatus::Pending,
                        ..tx
                    };
                    self.pending_txs.insert(tx_hash.clone(), pending_tx);
                    return tx_hash;
                }
                MockFailureMode::Revert(reason) => {
                    let failed_tx = MockTransaction {
                        status: MockTxStatus::Failure(reason.clone()),
                        ..tx
                    };
                    self.pending_txs.insert(tx_hash.clone(), failed_tx);
                    return tx_hash;
                }
            }
        }

        // Process successful transaction
        let success_tx = MockTransaction {
            status: MockTxStatus::Success,
            ..tx
        };
        self.pending_txs.insert(tx_hash.clone(), success_tx);
        tx_hash
    }

    pub fn get_transaction_status(&self, tx_hash: &str) -> Option<MockChainResponse> {
        if let Some(tx) = self.pending_txs.get(tx_hash) {
            match &tx.status {
                MockTxStatus::Success => {
                    Some(MockChainResponse::Success {
                        tx_hash: tx_hash.to_string(),
                        block_number: self.current_block,
                        gas_used: tx.gas_used,
                    })
                }
                MockTxStatus::Failure(error) => {
                    Some(MockChainResponse::Failure {
                        error: error.clone(),
                        revert_reason: error.clone(),
                    })
                }
                MockTxStatus::Pending => {
                    Some(MockChainResponse::Pending {
                        tx_hash: tx_hash.to_string(),
                        confirmations: 0,
                    })
                }
            }
        } else {
            None
        }
    }

    pub fn is_transaction_finalized(&self, tx_hash: &str) -> bool {
        if let Some(tx) = self.pending_txs.get(tx_hash) {
            // Check if we have enough confirmations
            let confirmations_needed = self.config.finality_blocks;
            let current_confirmations = self.current_block.saturating_sub(tx.gas_used as u64);
            current_confirmations >= confirmations_needed as u64
        } else {
            false
        }
    }

    pub fn get_bridge_contract_address(&self) -> String {
        self.config.bridge_contract.clone()
    }

    pub fn simulate_cross_chain_call(
        &mut self,
        target_chain: u32,
        payload: &[u8],
    ) -> MockChainResponse {
        // Simulate network delay
        std::thread::sleep(std::time::Duration::from_millis(100));

        let tx = MockTransaction {
            hash: format!("cross_chain_tx_{}_{}", self.config.chain_id, rand::random::<u64>()),
            from: self.config.bridge_contract.clone(),
            to: format!("bridge_{}", target_chain),
            value: "0".to_string(),
            data: hex::encode(payload),
            gas_used: 21000,
            status: MockTxStatus::Success,
        };

        let tx_hash = self.submit_transaction(tx);
        self.advance_blocks(self.config.finality_blocks as u64);
        self.get_transaction_status(&tx_hash).unwrap()
    }
}

/// Mock chain manager for managing multiple chains
pub struct MockChainManager {
    chains: HashMap<u32, MockChain>,
}

impl MockChainManager {
    pub fn new() -> Self {
        let mut chains = HashMap::new();
        chains.insert(1, MockChain::new(1, "Stellar"));
        chains.insert(2, MockChain::new(2, "Ethereum"));
        chains.insert(3, MockChain::new(3, "Polygon"));
        chains.insert(4, MockChain::new(4, "BSC"));
        
        Self { chains }
    }

    pub fn get_chain(&mut self, chain_id: u32) -> &mut MockChain {
        self.chains.get_mut(&chain_id).unwrap()
    }

    pub fn set_global_failure_mode(&mut self, mode: MockFailureMode) {
        for chain in self.chains.values_mut() {
            chain.set_failure_mode(mode.clone());
        }
    }

    pub fn clear_global_failure_mode(&mut self) {
        for chain in self.chains.values_mut() {
            chain.clear_failure_mode();
        }
    }

    pub fn advance_all_chains(&mut self, blocks: u64) {
        for chain in self.chains.values_mut() {
            chain.advance_blocks(blocks);
        }
    }

    pub fn simulate_cross_chain_message(
        &mut self,
        source_chain: u32,
        target_chain: u32,
        message: &[u8],
    ) -> Result<MockChainResponse, String> {
        let source = self.get_chain(source_chain);
        let target = self.get_chain(target_chain);

        // Submit transaction on source chain
        let tx = MockTransaction {
            hash: format!("msg_tx_{}_{}", source_chain, rand::random::<u64>()),
            from: format!("user_{}", source_chain),
            to: target.get_bridge_contract_address(),
            value: "0".to_string(),
            data: hex::encode(message),
            gas_used: 50000,
            status: MockTxStatus::Success,
        };

        let tx_hash = source.submit_transaction(tx);
        
        // Simulate message relay
        match source.get_transaction_status(&tx_hash) {
            Some(MockChainResponse::Success { .. }) => {
                // Relay to target chain
                let response = target.simulate_cross_chain_call(source_chain, message);
                Ok(response)
            }
            Some(MockChainResponse::Failure { error, .. }) => {
                Err(error)
            }
            _ => Err("Transaction not found".to_string()),
        }
    }
}
