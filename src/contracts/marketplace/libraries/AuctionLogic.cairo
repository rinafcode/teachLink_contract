pub mod AuctionLogic {
    use starknet::ContractAddress;

    #[derive(Drop, Serde, starknet::Store, Clone, Copy)]
    pub struct MinBidResult { pub min_next_bid: u256 }

    pub fn compute_min_next_bid(current_bid: u256, min_increment_bps: u16) -> MinBidResult {
        if current_bid == 0 { return MinBidResult { min_next_bid: 0 }; }
        let inc = (current_bid * min_increment_bps.into()) / 10000;
        MinBidResult { min_next_bid: current_bid + inc }
    }

    pub fn current_dutch_price(start_price: u256, end_price: u256, start_time: u64, end_time: u64, now: u64) -> u256 {
        if now >= end_time { return end_price; }
        if now <= start_time { return start_price; }
        // linear interpolation
        let total = (end_time - start_time).into();
        let elapsed = (now - start_time).into();
        let diff = start_price - end_price;
        let dec = diff * elapsed / total;
        start_price - dec
    }
}


