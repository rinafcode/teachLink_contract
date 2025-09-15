
            // Verify commitment
            let commitment = self.commitments.read(caller);
            let computed_hash = (amount_in + nonce) % (2_u256.pow(256) - 1);
            assert(commitment == computed_hash, Errors::INVALID_COMMITMENT);
            
            // Clear commitment
            self.commitments.write(caller, 0);
            self.commitment_blocks.write(caller, 0);
            
            self.emit(TradeRevealed { user: caller, amount: amount_in, nonce });
            
            // Execute the trade
            self.swap_exact_tokens_for_tokens(amount_in, amount_out_min, token_in, to, deadline)
        }
        
        fn add_to_batch_auction(
            ref self: ContractState,
            amount: u256,
            limit_price: u256,
            is_buy: bool
        ) {
            assert(self.batch_auction_enabled.read(), 'Batch auction disabled');
            
            let current_time = get_block_timestamp();
            let batch_end = self.current_batch_end.read();
            
            if current_time >= batch_end {
                // Start new batch
                let new_batch_id = self.batch_counter.read() + 1;
                self.batch_counter.write(new_batch_id);
                self.current_batch_end.write(current_time + self.batch_duration.read());
            }
            
            let batch_id = self.batch_counter.read();
            let caller = get_caller_address();
            
            if is_buy {
                let mut buy_orders = self.buy_orders.read(batch_id);
                buy_orders.append((caller, amount, limit_price));
                self.buy_orders.write(batch_id, buy_orders);
            } else {
                let mut sell_orders = self.sell_orders.read(batch_id);
                sell_orders.append((caller, amount, limit_price));
                self.sell_orders.write(batch_id, sell_orders);
            }
        }
        
        fn execute_batch_auction(ref self: ContractState) -> (u256, u256) {
            self.ownable.assert_only_owner();
            
            let current_time = get_block_timestamp();
            let batch_end = self.current_batch_end.read();
            assert(current_time >= batch_end, 'Batch still active');
            
            let batch_id = self.batch_counter.read();
            let buy_orders = self.buy_orders.read(batch_id);
            let sell_orders = self.sell_orders.read(batch_id);
            
            // Convert to format expected by AMM calculation
            let mut buy_orders_formatted: Array<(u256, u256)> = ArrayTrait::new();
            let mut sell_orders_formatted: Array<(u256, u256)> = ArrayTrait::new();
            
            let mut i = 0;
            while i < buy_orders.len() {
                let (_, amount, price) = *buy_orders.at(i);
                buy_orders_formatted.append((amount, price));
                i += 1;
            }
            
            i = 0;
            while i < sell_orders.len() {
                let (_, amount, price) = *sell_orders.at(i);
                sell_orders_formatted.append((amount, price));
                i += 1;
            }
            
            let (clearing_price, volume) = AMM::calculate_batch_clearing_price(
                buy_orders_formatted, 
                sell_orders_formatted
            );
            
            // Execute matched orders (simplified)
            self._execute_matched_orders(batch_id, clearing_price, volume);
            
            let participants = buy_orders.len() + sell_orders.len();
            self.emit(BatchAuctionExecuted { 
                batch_id, 
                clearing_price, 
                volume, 
                participants: participants.into() 
            });
            
            (clearing_price, volume)
        }
        
        fn set_mev_protection_params(
            ref self: ContractState,
            max_price_impact: u256,
            sandwich_protection_window: u64,
            volume_threshold: u256,
            consecutive_trade_limit: u256,
            flash_loan_protection: bool
        ) {
            self.ownable.assert_only_owner();
            
            let mev_protection = MEVProtectionData {
                max_price_impact,
                sandwich_protection_window,
                volume_threshold,
                consecutive_trade_limit,
                flash_loan_protection,
            };
            
            self.mev_protection.write(mev_protection);
        }
        
        fn enable_circuit_breaker(ref self: ContractState, enabled: bool) {
            self.ownable.assert_only_owner();
            self.circuit_breaker_enabled.write(enabled);
        }
        
        fn enable_batch_auction(ref self: ContractState, enabled: bool) {
            self.ownable.assert_only_owner();
            self.batch_auction_enabled.write(enabled);
            
            if enabled && self.current_batch_end.read() == 0 {
                let current_time = get_block_timestamp();
                self.current_batch_end.write(current_time + self.batch_duration.read());
            }
        }
        
        fn mark_suspicious_address(ref self: ContractState, address: ContractAddress, suspicious: bool) {
            self.ownable.assert_only_owner();
            self.suspicious_addresses.write(address, suspicious);
        }
        
        fn get_twap_price(self: @ContractState) -> (u256, u256) {
            let twap_data = self.twap_data.read();
            (twap_data.price_average_0, twap_data.price_average_1)
        }
        
        fn get_current_price_impact(self: @ContractState, amount_in: u256, token_in: ContractAddress) -> u256 {
            let reserve_a = self.reserve_a.read();
            let reserve_b = self.reserve_b.read();
            let fee_rate = self.fee_rate.read();
            
            let (reserve_in, reserve_out) = if token_in == self.token_a.read() {
                (reserve_a, reserve_b)
            } else {
                (reserve_b, reserve_a)
            };
            
            AMM::calculate_price_impact(amount_in, reserve_in, reserve_out, fee_rate)
        }
        
        fn is_circuit_breaker_active(self: @ContractState) -> bool {
            let current_time = get_block_timestamp();
            let breaker_end = self.circuit_breaker_end_time.read();
            
            self.circuit_breaker_enabled.read() && current_time < breaker_end
        }
    }
    
    #[abi(embed_v0)]
    impl ILProtectionManagement<ContractState> {
        fn set_il_protection_parameters(
            ref self: ContractState,
            threshold: u256,
            coverage: u256,
            duration: u64,
            oracle: ContractAddress
        ) {
            self.ownable.assert_only_owner();
            assert(threshold <= 10000, 'Invalid threshold'); // Max 100%
            assert(coverage <= 10000, 'Invalid coverage');   // Max 100%
            
            self.il_protection_threshold.write(threshold);
            self.il_protection_coverage.write(coverage);
            self.il_protection_duration.write(duration);
            self.oracle_address.write(oracle);
        }
        
        fn fund_il_protection(ref self: ContractState, amount: u256) {
            self.ownable.assert_only_owner();
            
            let reward_token = IERC20Dispatcher { contract_address: self.reward_token.read() };
            reward_token.transfer_from(get_caller_address(), get_contract_address(), amount);
            
            let current_fund = self.il_protection_fund.read();
            self.il_protection_fund.write(current_fund + amount);
        }
        
        fn get_il_protection_info(self: @ContractState, user: ContractAddress) -> (bool, u256, u256, u256) {
            let enabled = self.il_protection_enabled.read(user);
            let initial_value = self.initial_deposit_value.read(user);
            let available_compensation = self.calculate_il_compensation(user);
            let fund_balance = self.il_protection_fund.read();
            
            (enabled, initial_value, available_compensation, fund_balance)
        }
        