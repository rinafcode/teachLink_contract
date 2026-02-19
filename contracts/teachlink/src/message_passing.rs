//! Cross-Chain Message Passing Module
//!
//! This module implements guaranteed message delivery between chains
//! with retry mechanisms and timeout handling.

use crate::errors::BridgeError;
use crate::events::{PacketDeliveredEvent, PacketFailedEvent, PacketSentEvent};
use crate::storage::{CROSS_CHAIN_PACKETS, MESSAGE_RECEIPTS, PACKET_COUNTER};
use crate::types::{CrossChainPacket, MessageReceipt, PacketStatus};
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
        // Validate addresses
        if sender.is_empty() || sender.len() > 64 {
            return Err(BridgeError::InvalidPayload);
        }
        if recipient.is_empty() || recipient.len() > 64 {
            return Err(BridgeError::InvalidPayload);
        }
        if payload.is_empty() {
            return Err(BridgeError::InvalidPayload);
        }

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
