#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, symbol_short, Address, Bytes, BytesN,
    Env,
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

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IdentityRegistryError {
    DidAlreadyExists = 1,
    DidNotFound = 2,
    Unauthorized = 3,
    RecoveryNotConfigured = 4,
}

#[contractimpl]
impl IdentityRegistryContract {
    // Create a new DID mapping to a controller address.
    pub fn create_did(
        env: &Env,
        identity_id: BytesN<32>,
        controller: Address,
    ) -> Result<(), IdentityRegistryError> {
        controller.require_auth();
        let key = (symbol_short!("didctl"), identity_id.clone());
        if env.storage().persistent().has(&key) {
            return Err(IdentityRegistryError::DidAlreadyExists);
        }
        env.storage().persistent().set(&key, &controller);
        Didcrt {
            identity_id,
            controller,
        }
        .publish(env);
        Ok(())
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
    ) -> Result<(), IdentityRegistryError> {
        current_controller.require_auth();
        let key = (symbol_short!("didctl"), identity_id.clone());
        let opt: Option<Address> = env.storage().persistent().get(&key);
        match opt {
            Some(stored) => {
                if stored != current_controller {
                    return Err(IdentityRegistryError::Unauthorized);
                }
                env.storage().persistent().set(&key, &new_controller);
                Didchg {
                    identity_id,
                    new_controller,
                }
                .publish(env);
                Ok(())
            }
            None => Err(IdentityRegistryError::DidNotFound),
        }
    }

    // Add or update an authentication method (e.g., key, service) for a DID
    pub fn set_auth_method(
        env: &Env,
        identity_id: BytesN<32>,
        controller: Address,
        method_id: Bytes,
        public_key: Bytes,
    ) -> Result<(), IdentityRegistryError> {
        controller.require_auth();
        let ctrl_key = (symbol_short!("didctl"), identity_id.clone());
        let current: Option<Address> = env.storage().persistent().get(&ctrl_key);
        let current = current.ok_or(IdentityRegistryError::DidNotFound)?;
        if current != controller {
            return Err(IdentityRegistryError::Unauthorized);
        }
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
        Ok(())
    }

    // Remove an auth method
    pub fn remove_auth_method(
        env: &Env,
        identity_id: BytesN<32>,
        controller: Address,
        method_id: Bytes,
    ) -> Result<(), IdentityRegistryError> {
        controller.require_auth();
        let ctrl_key = (symbol_short!("didctl"), identity_id.clone());
        let current: Option<Address> = env.storage().persistent().get(&ctrl_key);
        let current = current.ok_or(IdentityRegistryError::DidNotFound)?;
        if current != controller {
            return Err(IdentityRegistryError::Unauthorized);
        }
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
        Ok(())
    }

    // Set a recovery address that may be used to recover control of the DID
    pub fn set_recovery(
        env: &Env,
        identity_id: BytesN<32>,
        controller: Address,
        recovery: Address,
    ) -> Result<(), IdentityRegistryError> {
        controller.require_auth();
        let ctrl_key = (symbol_short!("didctl"), identity_id.clone());
        let current: Option<Address> = env.storage().persistent().get(&ctrl_key);
        let current = current.ok_or(IdentityRegistryError::DidNotFound)?;
        if current != controller {
            return Err(IdentityRegistryError::Unauthorized);
        }
        let key = (symbol_short!("recovery"), identity_id.clone());
        env.storage().persistent().set(&key, &recovery);
        RecoverySet {
            identity_id,
            recovery,
        }
        .publish(env);
        Ok(())
    }

    // Recover controller using the configured recovery address
    pub fn recover(
        env: &Env,
        identity_id: BytesN<32>,
        recovery: Address,
        new_controller: Address,
    ) -> Result<(), IdentityRegistryError> {
        recovery.require_auth();
        let rec_key = (symbol_short!("recovery"), identity_id.clone());
        let rec_opt: Option<Address> = env.storage().persistent().get(&rec_key);
        match rec_opt {
            Some(recovery_addr) => {
                if recovery_addr != recovery {
                    return Err(IdentityRegistryError::Unauthorized);
                }
                let ctrl_key = (symbol_short!("didctl"), identity_id.clone());
                env.storage().persistent().set(&ctrl_key, &new_controller);
                Didrec {
                    identity_id,
                    new_controller,
                }
                .publish(env);
                Ok(())
            }
            None => Err(IdentityRegistryError::RecoveryNotConfigured),
        }
    }
}
