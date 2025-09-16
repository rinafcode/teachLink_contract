use starknet::{ContractAddress, get_block_timestamp};

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub struct TimeWindow { pub start: u64, pub end: u64 }

#[derive(Drop, Serde, starknet::Store, Clone, Copy, PartialEq)]
pub enum CheckResult { Allowed: (), Denied: () }

#[generate_trait]
pub impl AccessControl of AccessControlTrait {
    fn is_within_time_window(window: TimeWindow, now: u64) -> bool {
        if window.start != 0 && now < window.start { return false; }
        if window.end != 0 && now > window.end { return false; }
        true
    }

    fn compute_expiry(current: u64, duration: u64, max_end: u64) -> u64 {
        let mut exp = current + duration;
        if max_end != 0 && exp > max_end { exp = max_end; }
        exp
    }
}


