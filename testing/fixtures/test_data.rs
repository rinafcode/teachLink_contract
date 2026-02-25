/// Test data fixtures and generators
use soroban_sdk::{Address, Bytes, Env, String as SorobanString};

pub struct TestDataGenerator {
    env: Env,
    seed: u64,
}

impl TestDataGenerator {
    pub fn new(env: Env) -> Self {
        Self { env, seed: 12345 }
    }

    pub fn with_seed(env: Env, seed: u64) -> Self {
        Self { env, seed }
    }

    /// Generate test addresses
    pub fn addresses(&self, count: usize) -> Vec<Address> {
        (0..count).map(|_| Address::generate(&self.env)).collect()
    }

    /// Generate test amounts within range
    pub fn amounts(&self, min: i128, max: i128, count: usize) -> Vec<i128> {
        let range = max - min;
        (0..count)
            .map(|i| min + ((i as i128 * 1234567) % range))
            .collect()
    }

    /// Generate test strings
    pub fn strings(&self, prefix: &str, count: usize) -> Vec<SorobanString> {
        (0..count)
            .map(|i| SorobanString::from_str(&self.env, &format!("{}_{}", prefix, i)))
            .collect()
    }

    /// Generate test bytes
    pub fn bytes(&self, length: usize, count: usize) -> Vec<Bytes> {
        (0..count)
            .map(|i| {
                let data: Vec<u8> = (0..length).map(|j| ((i + j) % 256) as u8).collect();
                Bytes::from_slice(&self.env, &data)
            })
            .collect()
    }

    /// Generate cross-chain addresses (20-32 bytes)
    pub fn cross_chain_addresses(&self, count: usize) -> Vec<Bytes> {
        (0..count)
            .map(|i| {
                let length = 20 + (i % 13); // 20-32 bytes
                let data: Vec<u8> = (0..length).map(|j| ((i + j) % 256) as u8).collect();
                Bytes::from_slice(&self.env, &data)
            })
            .collect()
    }

    /// Generate chain IDs
    pub fn chain_ids(&self, count: usize) -> Vec<u32> {
        (1..=count as u32).collect()
    }

    /// Generate timestamps
    pub fn timestamps(&self, start: u64, interval: u64, count: usize) -> Vec<u64> {
        (0..count)
            .map(|i| start + (i as u64 * interval))
            .collect()
    }
}

/// Common test fixtures
pub struct TestFixtures;

impl TestFixtures {
    /// Standard test amounts
    pub fn standard_amounts() -> Vec<i128> {
        vec![
            1,
            100,
            1_000,
            10_000,
            100_000,
            1_000_000,
            10_000_000,
        ]
    }

    /// Edge case amounts
    pub fn edge_case_amounts() -> Vec<i128> {
        vec![
            0,
            1,
            i128::MAX / 2,
            -1,
        ]
    }

    /// Standard chain IDs
    pub fn standard_chain_ids() -> Vec<u32> {
        vec![
            1,  // Ethereum
            56, // BSC
            137, // Polygon
            43114, // Avalanche
        ]
    }

    /// Standard timeouts (in seconds)
    pub fn standard_timeouts() -> Vec<u64> {
        vec![
            60,      // 1 minute
            300,     // 5 minutes
            3600,    // 1 hour
            86400,   // 1 day
            604800,  // 1 week
        ]
    }

    /// Standard thresholds
    pub fn standard_thresholds() -> Vec<(u32, u32)> {
        vec![
            (1, 1),  // 1 of 1
            (1, 2),  // 1 of 2
            (2, 3),  // 2 of 3
            (3, 5),  // 3 of 5
            (5, 7),  // 5 of 7
        ]
    }
}

/// Mock data builder
pub struct MockDataBuilder {
    env: Env,
}

impl MockDataBuilder {
    pub fn new(env: Env) -> Self {
        Self { env }
    }

    pub fn escrow_params(&self) -> EscrowTestData {
        EscrowTestData {
            depositor: Address::generate(&self.env),
            beneficiary: Address::generate(&self.env),
            token: Address::generate(&self.env),
            amount: 1000,
            threshold: 2,
            signers: vec![
                Address::generate(&self.env),
                Address::generate(&self.env),
                Address::generate(&self.env),
            ],
            release_time: None,
            refund_time: None,
            arbitrator: Address::generate(&self.env),
        }
    }

    pub fn bridge_params(&self) -> BridgeTestData {
        BridgeTestData {
            from: Address::generate(&self.env),
            amount: 1000,
            dest_chain: 1,
            dest_address: Bytes::from_array(&self.env, &[1u8; 20]),
        }
    }

    pub fn reward_params(&self) -> RewardTestData {
        RewardTestData {
            recipient: Address::generate(&self.env),
            amount: 500,
            reward_type: SorobanString::from_str(&self.env, "course_completion"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EscrowTestData {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub threshold: u32,
    pub signers: Vec<Address>,
    pub release_time: Option<u64>,
    pub refund_time: Option<u64>,
    pub arbitrator: Address,
}

#[derive(Debug, Clone)]
pub struct BridgeTestData {
    pub from: Address,
    pub amount: i128,
    pub dest_chain: u32,
    pub dest_address: Bytes,
}

#[derive(Debug, Clone)]
pub struct RewardTestData {
    pub recipient: Address,
    pub amount: i128,
    pub reward_type: SorobanString,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_generator() {
        let env = Env::default();
        let generator = TestDataGenerator::new(env);
        
        let addresses = generator.addresses(5);
        assert_eq!(addresses.len(), 5);
        
        let amounts = generator.amounts(100, 1000, 10);
        assert_eq!(amounts.len(), 10);
        assert!(amounts.iter().all(|&a| a >= 100 && a < 1000));
    }

    #[test]
    fn test_fixtures() {
        let amounts = TestFixtures::standard_amounts();
        assert!(amounts.len() > 0);
        
        let chain_ids = TestFixtures::standard_chain_ids();
        assert!(chain_ids.contains(&1));
    }
}
