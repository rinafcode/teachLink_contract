#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, Map};

/// TeachLink main contract.
#[contract]
pub struct TeachLinkBridge;

#[contractimpl]
impl TeachLinkBridge {
    /// Initialize the bridge contract
    pub fn initialize(
        env: Env,
        token: Address,
        admin: Address,
        min_validators: u32,
        fee_recipient: Address,
    ) {
        env.storage()
            .instance()
            .set(&symbol_short!("token"), &token);
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("min_val"), &min_validators);
        env.storage()
            .instance()
            .set(&symbol_short!("fee_rec"), &fee_recipient);
        env.storage().instance().set(&symbol_short!("nonce"), &0u64);
    }

    /// Bridge tokens out to another chain
    pub fn bridge_out(
        env: Env,
        from: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> u64 {
        let nonce = env
            .storage()
            .instance()
            .get(&symbol_short!("nonce"))
            .unwrap_or(0u64);
        let new_nonce = nonce + 1;
        env.storage()
            .instance()
            .set(&symbol_short!("nonce"), &new_nonce);

        // Store bridge transaction
        let key = symbol_short!("bridge");
        let mut bridge_txs = env.storage().instance().get(&key).unwrap_or(Map::new(&env));
        bridge_txs.set(
            new_nonce,
            (from, amount, destination_chain, destination_address),
        );
        env.storage().instance().set(&key, &bridge_txs);

        new_nonce
    }

    /// Get nonce
    pub fn get_nonce(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&symbol_short!("nonce"))
            .unwrap_or(0u64)
    }

    /// Get token address
    pub fn get_token(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("token"))
            .unwrap()
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap()
    }
}
