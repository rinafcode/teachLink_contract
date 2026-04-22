//! Cross-Chain Message Passing Module
//!
//! This module implements guaranteed message delivery between chains
//! with retry mechanisms and timeout handling.

use crate::errors::BridgeError;
use crate::events::{PacketDeliveredEvent, PacketFailedEvent, PacketSentEvent};
use crate::storage::{
    CROSS_CHAIN_PACKETS, MESSAGE_RECEIPTS, PACKET_COUNTER, PACKET_LAST_RETRY, PACKET_RETRY_COUNTS,
};
use crate::types::{CrossChainPacket, MessageReceipt, PacketStatus};
use crate::validation::NumberValidator;
use soroban_sdk::{Bytes, Env, Map, Vec};

/// Default packet timeout (24 hours)
pub const DEFAULT_PACKET_TIMEOUT: u64 = 86_400;

/// Maximum retry attempts
pub const MAX_RETRY_ATTEMPTS: u32 = 5;

/// Retry delay in seconds (exponential backoff)
pub const RETRY_DELAY_BASE: u64 = 300; // 5 minutes

/// Message Passing Manager
pub struct MessagePassing;

impl MessagePassing {
    /// Send a cross-chain packet
    pub fn send_packet(
        env: &Env,
        source_chain: u32,
        destination_chain: u32,
        sender: Bytes,
        recipient: Bytes,
        payload: Bytes,
        timeout: Option<u64>,
    ) -> Result<u64, BridgeError> {
        // Validate chain IDs
        NumberValidator::validate_chain_id(source_chain).map_err(|_| BridgeError::InvalidInput)?;
        NumberValidator::validate_chain_id(destination_chain)
            .map_err(|_| BridgeError::InvalidInput)?;

        // Validate addresses (1–64 bytes)
        crate::validation::BytesValidator::validate_length(&sender, 1, 64)
            .map_err(|_| BridgeError::InvalidPayload)?;
        crate::validation::BytesValidator::validate_length(&recipient, 1, 64)
            .map_err(|_| BridgeError::InvalidPayload)?;

        // Validate payload (non-empty, within size limit)
        crate::validation::BytesValidator::validate_payload(&payload)
            .map_err(|_| BridgeError::InvalidPayload)?;

        // Get packet counter
        let mut packet_counter: u64 = env
            .storage()
            .instance()
            .get(&PACKET_COUNTER)
            .unwrap_or(0u64);
        packet_counter += 1;

        let packet_timeout = timeout.unwrap_or(DEFAULT_PACKET_TIMEOUT);

        // Create packet
        let packet = CrossChainPacket {
            packet_id: packet_counter,
            source_chain,
            destination_chain,
            sender: sender.clone(),
            recipient: recipient.clone(),
            payload: payload.clone(),
            nonce: packet_counter,
            timeout: env.ledger().timestamp() + packet_timeout,
            status: PacketStatus::Pending,
        };

        // Store packet
        let mut packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));
        packets.set(packet_counter, packet);
        env.storage().instance().set(&CROSS_CHAIN_PACKETS, &packets);
        env.storage()
            .instance()
            .set(&PACKET_COUNTER, &packet_counter);

        let mut retry_counts: Map<u64, u32> = env
            .storage()
            .instance()
            .get(&PACKET_RETRY_COUNTS)
            .unwrap_or_else(|| Map::new(env));
        retry_counts.set(packet_counter, 0);
        env.storage()
            .instance()
            .set(&PACKET_RETRY_COUNTS, &retry_counts);

        let mut last_retry: Map<u64, u64> = env
            .storage()
            .instance()
            .get(&PACKET_LAST_RETRY)
            .unwrap_or_else(|| Map::new(env));
        last_retry.set(packet_counter, env.ledger().timestamp());
        env.storage()
            .instance()
            .set(&PACKET_LAST_RETRY, &last_retry);

        // Emit event
        PacketSentEvent {
            packet_id: packet_counter,
            source_chain,
            destination_chain,
            sender,
            nonce: packet_counter,
        }
        .publish(env);

        Ok(packet_counter)
    }

    /// Mark a packet as delivered
    pub fn deliver_packet(
        env: &Env,
        packet_id: u64,
        gas_used: u64,
        result: Bytes,
    ) -> Result<(), BridgeError> {
        // Get packet
        let mut packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));
        let mut packet = packets.get(packet_id).ok_or(BridgeError::PacketNotFound)?;

        // Check if already delivered or failed
        match packet.status {
            PacketStatus::Delivered => return Err(BridgeError::SwapAlreadyCompleted),
            PacketStatus::Failed => return Err(BridgeError::InvalidInput),
            PacketStatus::TimedOut => return Err(BridgeError::PacketTimeout),
            _ => {}
        }

        // Check timeout
        if env.ledger().timestamp() > packet.timeout {
            packet.status = PacketStatus::TimedOut;
            packets.set(packet_id, packet);
            env.storage().instance().set(&CROSS_CHAIN_PACKETS, &packets);
            return Err(BridgeError::PacketTimeout);
        }

        // Mark as delivered
        packet.status = PacketStatus::Delivered;
        packets.set(packet_id, packet.clone());
        env.storage().instance().set(&CROSS_CHAIN_PACKETS, &packets);

        // Create receipt
        let receipt = MessageReceipt {
            packet_id,
            delivered_at: env.ledger().timestamp(),
            gas_used,
            result: result.clone(),
        };

        let mut receipts: Map<u64, MessageReceipt> = env
            .storage()
            .instance()
            .get(&MESSAGE_RECEIPTS)
            .unwrap_or_else(|| Map::new(env));
        receipts.set(packet_id, receipt);
        env.storage().instance().set(&MESSAGE_RECEIPTS, &receipts);

        // Emit event
        PacketDeliveredEvent {
            packet_id,
            delivered_at: env.ledger().timestamp(),
            gas_used,
        }
        .publish(env);

        Ok(())
    }

    /// Mark a packet as failed
    pub fn fail_packet(env: &Env, packet_id: u64, reason: Bytes) -> Result<(), BridgeError> {
        // Get packet
        let mut packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));
        let mut packet = packets.get(packet_id).ok_or(BridgeError::PacketNotFound)?;

        // Mark as failed
        packet.status = PacketStatus::Failed;
        packets.set(packet_id, packet);
        env.storage().instance().set(&CROSS_CHAIN_PACKETS, &packets);

        // Emit event
        PacketFailedEvent {
            packet_id,
            reason,
            failed_at: env.ledger().timestamp(),
        }
        .publish(env);

        Ok(())
    }

    /// Retry a failed packet
    pub fn retry_packet(env: &Env, packet_id: u64) -> Result<(), BridgeError> {
        // Get packet
        let mut packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));
        let mut packet = packets.get(packet_id).ok_or(BridgeError::PacketNotFound)?;

        // Only retry failed or timed out packets
        match packet.status {
            PacketStatus::Failed | PacketStatus::TimedOut => {
                let mut retry_counts: Map<u64, u32> = env
                    .storage()
                    .instance()
                    .get(&PACKET_RETRY_COUNTS)
                    .unwrap_or_else(|| Map::new(env));
                let retry_count = retry_counts.get(packet_id).unwrap_or(0);
                if retry_count >= MAX_RETRY_ATTEMPTS {
                    return Err(BridgeError::RetryLimitExceeded);
                }

                let mut last_retry: Map<u64, u64> = env
                    .storage()
                    .instance()
                    .get(&PACKET_LAST_RETRY)
                    .unwrap_or_else(|| Map::new(env));
                let last_retry_at = last_retry.get(packet_id).unwrap_or(0);
                let backoff_multiplier = 1u64 << retry_count;
                let retry_delay = RETRY_DELAY_BASE.saturating_mul(backoff_multiplier);
                let next_allowed_retry = last_retry_at.saturating_add(retry_delay);

                if env.ledger().timestamp() < next_allowed_retry {
                    return Err(BridgeError::RetryBackoffActive);
                }

                let updated_retry_count = retry_count + 1;
                retry_counts.set(packet_id, updated_retry_count);
                env.storage()
                    .instance()
                    .set(&PACKET_RETRY_COUNTS, &retry_counts);
                last_retry.set(packet_id, env.ledger().timestamp());
                env.storage()
                    .instance()
                    .set(&PACKET_LAST_RETRY, &last_retry);

                // Reset status to pending with new timeout
                packet.status = PacketStatus::Retrying;
                packet.timeout = env.ledger().timestamp() + DEFAULT_PACKET_TIMEOUT;
                packets.set(packet_id, packet);
                env.storage().instance().set(&CROSS_CHAIN_PACKETS, &packets);
                Ok(())
            }
            _ => Err(BridgeError::InvalidInput),
        }
    }

    /// Check and timeout expired packets
    pub fn check_timeouts(env: &Env) -> Result<Vec<u64>, BridgeError> {
        let packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));

        let mut timed_out_packets = Vec::new(env);
        let current_time = env.ledger().timestamp();

        for (packet_id, mut packet) in packets.iter() {
            if packet.status == PacketStatus::Pending || packet.status == PacketStatus::Retrying {
                if current_time > packet.timeout {
                    packet.status = PacketStatus::TimedOut;
                    timed_out_packets.push_back(packet_id);
                }
            }
        }

        // Update packets
        if !timed_out_packets.is_empty() {
            let mut packets = packets;
            for packet_id in timed_out_packets.iter() {
                if let Some(mut packet) = packets.get(packet_id) {
                    packet.status = PacketStatus::TimedOut;
                    packets.set(packet_id, packet);
                }
            }
            env.storage().instance().set(&CROSS_CHAIN_PACKETS, &packets);
        }

        Ok(timed_out_packets)
    }

    /// Get packet by ID
    pub fn get_packet(env: &Env, packet_id: u64) -> Option<CrossChainPacket> {
        let packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));
        packets.get(packet_id)
    }

    /// Get packet receipt
    pub fn get_receipt(env: &Env, packet_id: u64) -> Option<MessageReceipt> {
        let receipts: Map<u64, MessageReceipt> = env
            .storage()
            .instance()
            .get(&MESSAGE_RECEIPTS)
            .unwrap_or_else(|| Map::new(env));
        receipts.get(packet_id)
    }

    /// Get packets by status
    pub fn get_packets_by_status(env: &Env, status: PacketStatus) -> Vec<u64> {
        let packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (packet_id, packet) in packets.iter() {
            if packet.status == status {
                result.push_back(packet_id);
            }
        }
        result
    }

    /// Get pending packets for a chain
    pub fn get_pending_packets_for_chain(env: &Env, destination_chain: u32) -> Vec<u64> {
        let packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (packet_id, packet) in packets.iter() {
            if packet.destination_chain == destination_chain
                && (packet.status == PacketStatus::Pending
                    || packet.status == PacketStatus::Retrying)
            {
                result.push_back(packet_id);
            }
        }
        result
    }

    /// Verify packet delivery
    pub fn verify_delivery(env: &Env, packet_id: u64) -> bool {
        if let Some(packet) = Self::get_packet(env, packet_id) {
            packet.status == PacketStatus::Delivered
        } else {
            false
        }
    }

    pub fn get_packet_retry_count(env: &Env, packet_id: u64) -> u32 {
        let retry_counts: Map<u64, u32> = env
            .storage()
            .instance()
            .get(&PACKET_RETRY_COUNTS)
            .unwrap_or_else(|| Map::new(env));
        retry_counts.get(packet_id).unwrap_or(0)
    }

    /// Get packet count
    pub fn get_packet_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&PACKET_COUNTER)
            .unwrap_or(0u64)
    }

    /// Get all packets for a sender
    pub fn get_packets_by_sender(env: &Env, sender: Bytes) -> Vec<u64> {
        let packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (packet_id, packet) in packets.iter() {
            if packet.sender == sender {
                result.push_back(packet_id);
            }
        }
        result
    }

    /// Get all packets for a recipient
    pub fn get_packets_by_recipient(env: &Env, recipient: Bytes) -> Vec<u64> {
        let packets: Map<u64, CrossChainPacket> = env
            .storage()
            .instance()
            .get(&CROSS_CHAIN_PACKETS)
            .unwrap_or_else(|| Map::new(env));

        let mut result = Vec::new(env);
        for (packet_id, packet) in packets.iter() {
            if packet.recipient == recipient {
                result.push_back(packet_id);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{MessagePassing, DEFAULT_PACKET_TIMEOUT, RETRY_DELAY_BASE};
    use crate::errors::BridgeError;
    use crate::types::PacketStatus;
    use crate::TeachLinkBridge;
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{Address, Bytes, Env};

    fn set_time(env: &Env, timestamp: u64) {
        env.ledger().with_mut(|ledger_info| {
            ledger_info.timestamp = timestamp;
        });
    }

    fn with_contract<T>(env: &Env, contract_id: &Address, f: impl FnOnce() -> T) -> T {
        env.as_contract(contract_id, f)
    }

    #[test]
    fn retry_respects_exponential_backoff() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());
        set_time(&env, 1_000);

        let sender = Bytes::from_slice(&env, b"sender");
        let recipient = Bytes::from_slice(&env, b"recipient");
        let payload = Bytes::from_slice(&env, b"payload");

        let packet_id = with_contract(&env, &contract_id, || {
            MessagePassing::send_packet(&env, 1, 2, sender, recipient, payload, None)
                .expect("packet should be created")
        });

        with_contract(&env, &contract_id, || {
            MessagePassing::fail_packet(&env, packet_id, Bytes::from_slice(&env, b"relay error"))
                .expect("packet should be marked failed")
        });

        let first_retry_early = with_contract(&env, &contract_id, || {
            MessagePassing::retry_packet(&env, packet_id)
        });
        assert_eq!(first_retry_early, Err(BridgeError::RetryBackoffActive));

        set_time(&env, 1_000 + RETRY_DELAY_BASE);
        with_contract(&env, &contract_id, || {
            MessagePassing::retry_packet(&env, packet_id).expect("retry should succeed after delay")
        });
        let retry_count = with_contract(&env, &contract_id, || {
            MessagePassing::get_packet_retry_count(&env, packet_id)
        });
        assert_eq!(retry_count, 1);

        let packet = with_contract(&env, &contract_id, || {
            MessagePassing::get_packet(&env, packet_id).expect("packet should exist")
        });
        assert_eq!(packet.status, PacketStatus::Retrying);
        assert_eq!(
            packet.timeout,
            env.ledger().timestamp() + DEFAULT_PACKET_TIMEOUT
        );
    }

    #[test]
    fn retry_fails_after_max_attempts() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());
        set_time(&env, 2_000);

        let packet_id = with_contract(&env, &contract_id, || {
            MessagePassing::send_packet(
                &env,
                1,
                2,
                Bytes::from_slice(&env, b"sender"),
                Bytes::from_slice(&env, b"recipient"),
                Bytes::from_slice(&env, b"payload"),
                None,
            )
            .expect("packet should be created")
        });

        let mut now = 2_000u64;
        for retry_count in 0..5u32 {
            with_contract(&env, &contract_id, || {
                MessagePassing::fail_packet(&env, packet_id, Bytes::from_slice(&env, b"transient"))
                    .expect("packet should be failed before retry")
            });
            now += RETRY_DELAY_BASE * (1u64 << retry_count);
            set_time(&env, now);
            with_contract(&env, &contract_id, || {
                MessagePassing::retry_packet(&env, packet_id)
                    .expect("retry within limit should pass")
            });
        }

        with_contract(&env, &contract_id, || {
            MessagePassing::fail_packet(&env, packet_id, Bytes::from_slice(&env, b"final"))
                .expect("packet should be failed before final retry")
        });
        set_time(&env, now + 100_000);

        let retry_over_limit = with_contract(&env, &contract_id, || {
            MessagePassing::retry_packet(&env, packet_id)
        });
        assert_eq!(retry_over_limit, Err(BridgeError::RetryLimitExceeded));
    }

    #[test]
    fn check_timeouts_marks_packets_timed_out() {
        let env = Env::default();
        let contract_id = env.register(TeachLinkBridge, ());
        set_time(&env, 5_000);

        let packet_id = with_contract(&env, &contract_id, || {
            MessagePassing::send_packet(
                &env,
                1,
                2,
                Bytes::from_slice(&env, b"sender"),
                Bytes::from_slice(&env, b"recipient"),
                Bytes::from_slice(&env, b"payload"),
                Some(10),
            )
            .expect("packet should be created")
        });

        set_time(&env, 5_011);
        let timed_out = with_contract(&env, &contract_id, || {
            MessagePassing::check_timeouts(&env).expect("timeout check should succeed")
        });
        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out.get(0), Some(packet_id));

        let packet = with_contract(&env, &contract_id, || {
            MessagePassing::get_packet(&env, packet_id).expect("packet should exist")
        });
        assert_eq!(packet.status, PacketStatus::TimedOut);
    }
}
