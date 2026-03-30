//! Documentation versioning: track and update the global documentation version.

use soroban_sdk::Env;

use crate::storage::DocKey;

pub struct Versioning;

impl Versioning {
    /// Return the current documentation version.
    pub fn get(env: &Env) -> u32 {
        env.storage().instance().get(&DocKey::Version).unwrap_or(1)
    }

    /// Set a new documentation version.
    pub fn update(env: &Env, version: u32) {
        env.storage().instance().set(&DocKey::Version, &version);
    }
}
