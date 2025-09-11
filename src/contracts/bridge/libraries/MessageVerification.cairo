pub mod MessageVerification {
    use starknet::{ContractAddress, get_block_timestamp, storage::{StoragePointerReadAccess, StoragePointerWriteAccess}};
    use core::array::{Array, ArrayTrait};

    #[storage]
    struct Storage {
        // replay-protection for processed message ids (hashes)
        processed: felt252::LegacyMap<bool>,
        // trusted relayers
        trusted: felt252::LegacyMap<bool>,
        // simple rate limit: amount moved in the current window and window tracking
        window_start: u64,
        window_amount: u256,
        per_window_limit: u256,
        window_seconds: u64,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        TrustedRelayerUpdated: TrustedRelayerUpdated,
        RateLimitUpdated: RateLimitUpdated,
    }

    #[derive(Drop, starknet::Event)]
    pub struct TrustedRelayerUpdated { pub relayer: ContractAddress, pub trusted: bool }
    #[derive(Drop, starknet::Event)]
    pub struct RateLimitUpdated { pub per_window_limit: u256, pub window_seconds: u64 }

    #[generate_trait]
    impl LibImpl of LibTrait {
        fn is_trusted(self: @Storage, relayer: ContractAddress) -> bool {
            self.trusted.read(relayer.into()) == true
        }

        fn set_trusted(ref self: Storage, relayer: ContractAddress, trusted: bool) {
            self.trusted.write(relayer.into(), trusted);
            starknet::emit(Event::TrustedRelayerUpdated(TrustedRelayerUpdated { relayer, trusted }));
        }

        fn set_rate_limit(ref self: Storage, per_window_limit: u256, window_seconds: u64) {
            self.per_window_limit.write(per_window_limit);
            self.window_seconds.write(window_seconds);
            if self.window_start.read() == 0 { self.window_start.write(get_block_timestamp()); }
            starknet::emit(Event::RateLimitUpdated(RateLimitUpdated { per_window_limit, window_seconds }));
        }

        fn assert_and_consume_rate(ref self: Storage, amount: u256) {
            let now = get_block_timestamp();
            let start = self.window_start.read();
            let window = self.window_seconds.read();
            let limit = self.per_window_limit.read();
            if window == 0 { return; }
            if start == 0 || now >= start + window {
                self.window_start.write(now);
                self.window_amount.write(0);
            }
            let used = self.window_amount.read();
            let new_used = used + amount;
            assert(new_used <= limit, 'rate limit');
            self.window_amount.write(new_used);
        }

        fn is_processed(self: @Storage, message_id: felt252) -> bool {
            self.processed.read(message_id) == true
        }

        fn mark_processed(ref self: Storage, message_id: felt252) {
            self.processed.write(message_id, true);
        }
    }
}


