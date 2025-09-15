use core::integer::{u256_safe_div, u256_checked_mul, u256_overflow_mul};

#[derive(Drop, Copy)]
pub struct ReserveData {
    pub reserve_a: u256,
    pub reserve_b: u256,
    pub block_timestamp_last: u64,
}

#[derive(Drop, Copy, Serde, starknet::Store)]
pub struct TWAPData {
    pub price_0_cumulative_last: u256,
    pub price_1_cumulative_last: u256,
    pub block_timestamp_last: u64,
    pub price_average_0: u256,
    pub price_average_1: u256,
    pub observation_count: u256,
}

#[derive(Drop, Copy, Serde, starknet::Store)]
pub struct MEVProtectionData {
    pub max_price_impact: u256,        // Maximum allowed price impact (e.g., 500 = 5%)
    pub sandwich_protection_window: u64, // Time window for sandwich attack detection
    pub volume_threshold: u256,         // Volume threshold for large trade detection
    pub consecutive_trade_limit: u256,  // Max consecutive trades from same user
    pub flash_loan_protection: bool,   // Enable flash loan protection
}

pub mod AMM {
    use super::{ReserveData, TWAPData, MEVProtectionData};
    
    // Constant product formula: x * y = k
    pub fn get_amount_out(
        amount_in: u256,
        reserve_in: u256,
        reserve_out: u256,
        fee_rate: u256
    ) -> u256 {
        assert(amount_in > 0, 'INSUFFICIENT_INPUT_AMOUNT');
        assert(reserve_in > 0 && reserve_out > 0, 'INSUFFICIENT_LIQUIDITY');
        
        let amount_in_with_fee = amount_in * (10000 - fee_rate);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 10000 + amount_in_with_fee;
        
        numerator / denominator
    }
    
    pub fn get_amount_in(
        amount_out: u256,
        reserve_in: u256,
        reserve_out: u256,
        fee_rate: u256
    ) -> u256 {
        assert(amount_out > 0, 'INSUFFICIENT_OUTPUT_AMOUNT');
        assert(reserve_in > 0 && reserve_out > 0, 'INSUFFICIENT_LIQUIDITY');
        
        let numerator = reserve_in * amount_out * 10000;
        let denominator = (reserve_out - amount_out) * (10000 - fee_rate);
        
        (numerator / denominator) + 1
    }
    
    pub fn quote(amount_a: u256, reserve_a: u256, reserve_b: u256) -> u256 {
        assert(amount_a > 0, 'INSUFFICIENT_AMOUNT');
        assert(reserve_a > 0 && reserve_b > 0, 'INSUFFICIENT_LIQUIDITY');
        
        (amount_a * reserve_b) / reserve_a
    }
    
    // Calculate optimal liquidity amounts
    pub fn calculate_liquidity_amounts(
        amount_a_desired: u256,
        amount_b_desired: u256,
        reserve_a: u256,
        reserve_b: u256
    ) -> (u256, u256) {
        if reserve_a == 0 && reserve_b == 0 {
            return (amount_a_desired, amount_b_desired);
        }
        
        let amount_b_optimal = quote(amount_a_desired, reserve_a, reserve_b);
        if amount_b_optimal <= amount_b_desired {
            return (amount_a_desired, amount_b_optimal);
        }
        
        let amount_a_optimal = quote(amount_b_desired, reserve_b, reserve_a);
        assert(amount_a_optimal <= amount_a_desired, 'OPTIMAL_A_EXCEEDS_DESIRED');
        
        (amount_a_optimal, amount_b_desired)
    }
    
    // Calculate square root for liquidity calculation
    pub fn sqrt(y: u256) -> u256 {
        if y > 3 {
            let mut z = y;
            let mut x = y / 2 + 1;
            while x < z {
                z = x;
                x = (y / x + x) / 2;
            }
            z
        } else if y != 0 {
            1
        } else {
            0
        }
    }
    
    // Calculate minimum liquidity for first deposit
    pub const MINIMUM_LIQUIDITY: u256 = 1000;
    
    pub fn calculate_liquidity_minted(
        amount_a: u256,
        amount_b: u256,
        reserve_a: u256,
        reserve_b: u256,
        total_supply: u256
    ) -> u256 {
        if total_supply == 0 {
            sqrt(amount_a * amount_b) - MINIMUM_LIQUIDITY
        } else {
            let liquidity_a = (amount_a * total_supply) / reserve_a;
            let liquidity_b = (amount_b * total_supply) / reserve_b;
            
            if liquidity_a < liquidity_b {
                liquidity_a
            } else {
                liquidity_b
            }
        }
    }
    
    // MEV protection: Time-weighted average price
    pub fn update_twap(
        ref twap_data: TWAPData,
        reserve_a: u256,
        reserve_b: u256,
        current_timestamp: u64
    ) {
        let time_elapsed = current_timestamp - twap_data.block_timestamp_last;
        
        if time_elapsed > 0 && reserve_a != 0 && reserve_b != 0 {
            // Calculate current prices (fixed point arithmetic)
            let price_0 = (reserve_b * 1000000000000000000) / reserve_a;
            let price_1 = (reserve_a * 1000000000000000000) / reserve_b;
            
            // Update cumulative prices
            twap_data.price_0_cumulative_last += price_0 * time_elapsed.into();
            twap_data.price_1_cumulative_last += price_1 * time_elapsed.into();
            twap_data.block_timestamp_last = current_timestamp;
            twap_data.observation_count += 1;
            
            // Calculate moving averages (simplified)
            if twap_data.observation_count > 1 {
                twap_data.price_average_0 = twap_data.price_0_cumulative_last / twap_data.observation_count;
                twap_data.price_average_1 = twap_data.price_1_cumulative_last / twap_data.observation_count;
            }
        }
    }
    
    // Calculate price impact of a trade
    pub fn calculate_price_impact(
        amount_in: u256,
        reserve_in: u256,
        reserve_out: u256,
        fee_rate: u256
    ) -> u256 {
        if reserve_in == 0 || reserve_out == 0 {
            return 10000; // 100% impact if no liquidity
        }
        
        let amount_out = get_amount_out(amount_in, reserve_in, reserve_out, fee_rate);
        let price_before = (reserve_out * 1000000000000000000) / reserve_in;
        let new_reserve_in = reserve_in + amount_in;
        let new_reserve_out = reserve_out - amount_out;
        
        if new_reserve_out == 0 {
            return 10000; // 100% impact if depleting pool
        }
        
        let price_after = (new_reserve_out * 1000000000000000000) / new_reserve_in;
        
        if price_after >= price_before {
            return 0; // No negative impact
        }
        
        let price_change = price_before - price_after;
        (price_change * 10000) / price_before
    }
    
    // Detect potential sandwich attacks
    pub fn detect_sandwich_attack(
        current_price: u256,
        twap_price: u256,
        deviation_threshold: u256
    ) -> bool {
        if twap_price == 0 {
            return false;
        }
        
        let price_deviation = if current_price > twap_price {
            ((current_price - twap_price) * 10000) / twap_price
        } else {
            ((twap_price - current_price) * 10000) / twap_price
        };
        
        price_deviation > deviation_threshold
    }
    
    // Calculate dynamic fee based on volatility and volume
    pub fn calculate_dynamic_fee(
        base_fee: u256,
        price_volatility: u256,
        volume_factor: u256,
        max_fee: u256
    ) -> u256 {
        let volatility_adjustment = (price_volatility * 50) / 10000; // 0.5% per 1% volatility
        let volume_adjustment = (volume_factor * 20) / 10000; // 0.2% per volume factor
        
        let dynamic_fee = base_fee + volatility_adjustment + volume_adjustment;
        
        if dynamic_fee > max_fee {
            max_fee
        } else {
            dynamic_fee
        }
    }
    
    // Implement commit-reveal scheme for MEV protection
    pub fn verify_commit_reveal(
        commitment: u256,
        amount: u256,
        nonce: u256,
        revealed_hash: u256
    ) -> bool {
        // Simple hash verification (in production, use proper cryptographic hash)
        let computed_hash = (amount + nonce) % (2_u256.pow(256) - 1);
        commitment == computed_hash && revealed_hash == computed_hash
    }
    
    // Calculate fair price using multiple sources
    pub fn calculate_fair_price(
        pool_price: u256,
        twap_price: u256,
        oracle_price: u256,
        weights: (u256, u256, u256) // (pool_weight, twap_weight, oracle_weight)
    ) -> u256 {
        let (pool_weight, twap_weight, oracle_weight) = weights;
        let total_weight = pool_weight + twap_weight + oracle_weight;
        
        if total_weight == 0 {
            return pool_price;
        }
        
        let weighted_price = (pool_price * pool_weight + 
                            twap_price * twap_weight + 
                            oracle_price * oracle_weight) / total_weight;
        
        weighted_price
    }
    
    // Implement gradual price adjustment to prevent large swings
    pub fn calculate_gradual_price_adjustment(
        current_price: u256,
        target_price: u256,
        max_adjustment_per_block: u256
    ) -> u256 {
        if current_price == target_price {
            return current_price;
        }
        
        let price_diff = if target_price > current_price {
            target_price - current_price
        } else {
            current_price - target_price
        };
        
        let max_change = (current_price * max_adjustment_per_block) / 10000;
        
        if price_diff <= max_change {
            target_price
        } else {
            if target_price > current_price {
                current_price + max_change
            } else {
                current_price - max_change
            }
        }
    }
    
    // Calculate slippage protection
    pub fn calculate_slippage_protection(
        expected_amount: u256,
        actual_amount: u256,
        max_slippage: u256
    ) -> bool {
        if expected_amount == 0 {
            return false;
        }
        
        let slippage = if actual_amount < expected_amount {
            ((expected_amount - actual_amount) * 10000) / expected_amount
        } else {
            0
        };
        
        slippage <= max_slippage
    }
    
    // Implement batch auction mechanism
    pub fn calculate_batch_clearing_price(
        buy_orders: Array<(u256, u256)>, // (amount, max_price)
        sell_orders: Array<(u256, u256)>, // (amount, min_price)
    ) -> (u256, u256) { // (clearing_price, volume)
        // Simplified batch auction - in production, implement proper matching algorithm
        let mut total_buy_volume = 0;
        let mut total_sell_volume = 0;
        let mut weighted_buy_price = 0;
        let mut weighted_sell_price = 0;
        
        let mut i = 0;
        while i < buy_orders.len() {
            let (amount, price) = *buy_orders.at(i);
            total_buy_volume += amount;
            weighted_buy_price += amount * price;
            i += 1;
        }
        
        i = 0;
        while i < sell_orders.len() {
            let (amount, price) = *sell_orders.at(i);
            total_sell_volume += amount;
            weighted_sell_price += amount * price;
            i += 1;
        }
        
        let clearing_volume = if total_buy_volume < total_sell_volume {
            total_buy_volume
        } else {
            total_sell_volume
        };
        
        let avg_buy_price = if total_buy_volume > 0 {
            weighted_buy_price / total_buy_volume
        } else {
            0
        };
        
        let avg_sell_price = if total_sell_volume > 0 {
            weighted_sell_price / total_sell_volume
        } else {
            0
        };
        
        let clearing_price = (avg_buy_price + avg_sell_price) / 2;
        
        (clearing_price, clearing_volume)
    }
}
