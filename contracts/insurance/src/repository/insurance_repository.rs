//! Insurance Domain Repository
//! 
//! This module provides repository implementations for insurance-related data access.

use crate::repository::generic::{GenericCounterRepository, GenericMapRepository, SingleValueRepository};
use crate::repository::traits::InstanceStorage;
use crate::repository::StorageError;
use crate::storage::DataKey;
use crate::types::*;
use soroban_sdk::{Address, Env, Map, Vec};

/// Repository for insurance configuration
pub struct InsuranceConfigRepository<'a> {
    storage: InstanceStorage<'a>,
}

impl<'a> InsuranceConfigRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
        }
    }
    
    pub fn get_admin(&self) -> Result<Address, StorageError> {
        self.storage.get(&DataKey::Admin).ok_or(StorageError::NotFound)
    }
    
    pub fn set_admin(&self, admin: &Address) -> Result<(), StorageError> {
        self.storage.set(&DataKey::Admin, admin);
        Ok(())
    }
    
    pub fn get_oracle(&self) -> Result<Address, StorageError> {
        self.storage.get(&DataKey::Oracle).ok_or(StorageError::NotFound)
    }
    
    pub fn set_oracle(&self, oracle: &Address) -> Result<(), StorageError> {
        self.storage.set(&DataKey::Oracle, oracle);
        Ok(())
    }
    
    pub fn get_token(&self) -> Result<Address, StorageError> {
        self.storage.get(&DataKey::Token).ok_or(StorageError::NotFound)
    }
    
    pub fn set_token(&self, token: &Address) -> Result<(), StorageError> {
        self.storage.set(&DataKey::Token, token);
        Ok(())
    }
    
    pub fn is_initialized(&self) -> bool {
        self.storage.has(&DataKey::Admin)
    }
}

/// Repository for insurance policies
pub struct PolicyRepository<'a> {
    storage: InstanceStorage<'a>,
    counter: GenericCounterRepository<'a, DataKey>,
}

impl<'a> PolicyRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
            counter: GenericCounterRepository::new(&InstanceStorage::new(env), DataKey::PolicyCount),
        }
    }
    
    pub fn get_policy(&self, policy_id: u64) -> Option<Policy> {
        self.storage.get(&DataKey::Policy(policy_id))
    }
    
    pub fn save_policy(&self, policy: &Policy) -> Result<(), StorageError> {
        self.storage.set(&DataKey::Policy(policy.id), policy);
        Ok(())
    }
    
    pub fn get_next_id(&self) -> Result<u64, StorageError> {
        self.counter.increment()
    }
    
    pub fn get_policy_by_user(&self, user: &Address, course_id: u64) -> Option<u64> {
        self.storage.get(&DataKey::PolicyByUser(user.clone(), course_id))
    }
    
    pub fn get_active_policies(&self, user: &Address) -> Vec<u64> {
        self.storage.get(&DataKey::ActivePolicies(user.clone()))
            .unwrap_or_else(|| Vec::new(self.storage.env()))
    }
}

/// Repository for insurance claims
pub struct ClaimRepository<'a> {
    storage: InstanceStorage<'a>,
    counter: GenericCounterRepository<'a, DataKey>,
}

impl<'a> ClaimRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
            counter: GenericCounterRepository::new(&InstanceStorage::new(env), DataKey::ClaimCount),
        }
    }
    
    pub fn get_claim(&self, claim_id: u64) -> Option<Claim> {
        self.storage.get(&DataKey::Claim(claim_id))
    }
    
    pub fn save_claim(&self, claim: &Claim) -> Result<(), StorageError> {
        self.storage.set(&DataKey::Claim(claim.id), claim);
        Ok(())
    }
    
    pub fn get_next_id(&self) -> Result<u64, StorageError> {
        self.counter.increment()
    }
    
    pub fn get_pending_claims(&self) -> Vec<u64> {
        self.storage.get(&DataKey::PendingClaims).unwrap_or_else(|| Vec::new(self.storage.env()))
    }
}

/// Repository for insurance pools
pub struct PoolRepository<'a> {
    storage: InstanceStorage<'a>,
    counter: GenericCounterRepository<'a, DataKey>,
}

impl<'a> PoolRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
            counter: GenericCounterRepository::new(&InstanceStorage::new(env), DataKey::PoolCount),
        }
    }
    
    pub fn get_pool(&self, pool_id: u64) -> Option<Pool> {
        self.storage.get(&DataKey::Pool(pool_id))
    }
    
    pub fn save_pool(&self, pool: &Pool) -> Result<(), StorageError> {
        self.storage.set(&DataKey::Pool(pool.id), pool);
        Ok(())
    }
    
    pub fn get_next_id(&self) -> Result<u64, StorageError> {
        self.counter.increment()
    }
    
    pub fn get_active_pools(&self) -> Vec<u64> {
        self.storage.get(&DataKey::ActivePools).unwrap_or_else(|| Vec::new(self.storage.env()))
    }
}

/// Repository for risk profiles
pub struct RiskProfileRepository<'a> {
    storage: InstanceStorage<'a>,
    counter: GenericCounterRepository<'a, DataKey>,
}

impl<'a> RiskProfileRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            storage: InstanceStorage::new(env),
            counter: GenericCounterRepository::new(&InstanceStorage::new(env), DataKey::RiskProfileCount),
        }
    }
    
    pub fn get_risk_profile(&self, profile_id: u64) -> Option<RiskProfile> {
        self.storage.get(&DataKey::RiskProfile(profile_id))
    }
    
    pub fn save_risk_profile(&self, profile: &RiskProfile) -> Result<(), StorageError> {
        self.storage.set(&DataKey::RiskProfile(profile.id), profile);
        Ok(())
    }
    
    pub fn get_by_user(&self, user: &Address) -> Option<u64> {
        self.storage.get(&DataKey::RiskProfileByUser(user.clone()))
    }
    
    pub fn get_next_id(&self) -> Result<u64, StorageError> {
        self.counter.increment()
    }
}

/// Aggregate repository for all insurance operations
pub struct InsuranceRepository<'a> {
    pub config: InsuranceConfigRepository<'a>,
    pub policies: PolicyRepository<'a>,
    pub claims: ClaimRepository<'a>,
    pub pools: PoolRepository<'a>,
    pub risk_profiles: RiskProfileRepository<'a>,
}

impl<'a> InsuranceRepository<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self {
            config: InsuranceConfigRepository::new(env),
            policies: PolicyRepository::new(env),
            claims: ClaimRepository::new(env),
            pools: PoolRepository::new(env),
            risk_profiles: RiskProfileRepository::new(env),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    
    #[test]
    fn test_config_repository() {
        let env = Env::default();
        let repo = InsuranceConfigRepository::new(&env);
        
        let admin = Address::generate(&env);
        repo.set_admin(&admin).expect("Should set admin");
        
        assert_eq!(repo.get_admin().unwrap(), admin);
        assert!(repo.is_initialized());
    }
    
    #[test]
    fn test_policy_repository() {
        let env = Env::default();
        let repo = PolicyRepository::new(&env);
        
        let initial_count = repo.counter.get().expect("Should get count");
        assert_eq!(initial_count, 0);
        
        let next_id = repo.get_next_id().expect("Should get next ID");
        assert_eq!(next_id, 1);
    }
}
