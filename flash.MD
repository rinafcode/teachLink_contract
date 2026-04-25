use crate::types::*;
use soroban_sdk::{contracttype, Address, Vec};


pub struct QualityVote {
    pub token_id: u64,
    pub voter: Address,
    pub score: u8,
}


