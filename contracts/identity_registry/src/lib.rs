#![no_std]

use soroban_sdk::{
    contract, contractevent, contractimpl, symbol_short, Address, Bytes, BytesN, Env,
};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Didcrt {
    pub identity_id: BytesN<32>,
    pub controller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Didchg {
    pub identity_id: BytesN<32>,
    pub new_controller: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authset {
    pub identity_id: BytesN<32>,
    pub method_id: Bytes,
    pub public_key: Bytes,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Authrem {
    pub identity_id: BytesN<32>,
    pub method_id: Bytes,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoverySet {
    pub identity_id: BytesN<32>,
    pub recovery: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Didrec {
    pub identity_id: BytesN<32>,
    pub new_controller: Address,
}
#[contract]
pub struct IdentityRegistryContract;

#[contractimpl]
impl IdentityRegistryContract {
    // Create a new DID mapping to a controller address.
    pub fn create_did(env: &Env, identity_id: BytesN<32>, controller: Address) {
        controller.require_auth();
        let key = (symbol_short!("didctl"), identity_id.clone());
        assert!(!env.storage().persistent().has(&key), "DID already exists");
        env.storage().persistent().set(&key, &controller);
        Didcrt {
            identity_id,
            controller,
        }
        .publish(env);
    }

    // Get controller for a DID
    pub fn get_controller(env: &Env, identity_id: BytesN<32>) -> Option<Address> {
        let key = (symbol_short!("didctl"), identity_id.clone());
        env.storage().persistent().get(&key)
    }

    // Update controller. Caller must pass the current controller and sign the call.
    pub fn set_controller(
        env: &Env,
        identity_id: BytesN<32>,
        current_controller: Address,
        new_controller: Address,
    ) {
        current_controller.require_auth();
        let key = (symbol_short!("did_ctrl"), identity_id.clone());
        let opt: Option<Address> = env.storage().persistent().get(&key);
        match opt {
            Some(stored) => {
                assert!(
                    stored == current_controller,
                    "only controller can change controller"
                );
                env.storage().persistent().set(&key, &new_controller);
                Didchg {
                    identity_id,
                    new_controller,
                }
                .publish(env);
            }
            None => panic!("DID not found"),
        }
    }

    // Add or update an authentication method (e.g., key, service) for a DID
    pub fn set_auth_method(
        env: &Env,
        identity_id: BytesN<32>,
        controller: Address,
        method_id: Bytes,
        public_key: Bytes,
    ) {
        controller.require_auth();
        let ctrl_key = (symbol_short!("didctl"), identity_id.clone());
        let current: Option<Address> = env.storage().persistent().get(&ctrl_key);
        assert!(current.is_some(), "DID not found");
        assert!(
            current.unwrap() == controller,
            "only controller can set auth methods"
        );
        let key = (
            symbol_short!("auth"),
            identity_id.clone(),
            method_id.clone(),
        );
        env.storage().persistent().set(&key, &public_key);
        Authset {
            identity_id,
            method_id,
            public_key,
        }
        .publish(env);
    }

    // Remove an auth method
    pub fn remove_auth_method(
        env: &Env,
        identity_id: BytesN<32>,
        controller: Address,
        method_id: Bytes,
    ) {
        controller.require_auth();
        let ctrl_key = (symbol_short!("did_ctrl"), identity_id.clone());
        let current: Option<Address> = env.storage().persistent().get(&ctrl_key);
        assert!(current.is_some(), "DID not found");
        assert!(
            current.unwrap() == controller,
            "only controller can remove auth methods"
        );
        let key = (
            symbol_short!("auth"),
            identity_id.clone(),
            method_id.clone(),
        );
        env.storage().persistent().remove(&key);
        Authrem {
            identity_id,
            method_id,
        }
        .publish(env);
    }

    // Set a recovery address that may be used to recover control of the DID
    pub fn set_recovery(
        env: &Env,
        identity_id: BytesN<32>,
        controller: Address,
        recovery: Address,
    ) {
        controller.require_auth();
        let ctrl_key = (symbol_short!("did_ctrl"), identity_id.clone());
        let current: Option<Address> = env.storage().persistent().get(&ctrl_key);
        assert!(current.is_some(), "DID not found");
        assert!(
            current.unwrap() == controller,
            "only controller can set recovery"
        );
        let key = (symbol_short!("recovery"), identity_id.clone());
        env.storage().persistent().set(&key, &recovery);
        RecoverySet {
            identity_id,
            recovery,
        }
        .publish(env);
    }

    // Recover controller using the configured recovery address
    pub fn recover(env: &Env, identity_id: BytesN<32>, recovery: Address, new_controller: Address) {
        recovery.require_auth();
        let rec_key = (symbol_short!("recovery"), identity_id.clone());
        let rec_opt: Option<Address> = env.storage().persistent().get(&rec_key);
        match rec_opt {
            Some(recovery_addr) => {
                assert!(
                    recovery_addr == recovery,
                    "only recovery address can perform recovery"
                );
                let ctrl_key = (symbol_short!("did_ctrl"), identity_id.clone());
                env.storage().persistent().set(&ctrl_key, &new_controller);
                Didrec {
                    identity_id,
                    new_controller,
                }
                .publish(env);
            }
            None => panic!("no recovery configured for DID"),
        }
    }
}

fn main() {}
