// File: network.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// P2P networking primitives for Tetcore consensus and transaction propagation.
// Includes peer management, message types, gossip protocols, and connection state.

use crate::consensus::{FinalitySignature, Proposal, VoteMessage};
use crate::Address;
use crate::Hash32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerRole {
    Validator,
    FullNode,
    LightNode,
    Unknown,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Handshake,
    Connected,
    Banned,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub address: NetworkAddress,
    pub role: PeerRole,
    pub state: ConnectionState,
    pub protocol_version: u32,
    pub chain_id: u32,
    pub best_block: Option<(u64, Hash32)>,
    pub latency_ms: u64,
    pub score: f64,
    pub last_message_time: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl PeerInfo {
    pub fn new(peer_id: PeerId, address: NetworkAddress) -> Self {
        Self {
            peer_id,
            address,
            role: PeerRole::Unknown,
            state: ConnectionState::Disconnected,
            protocol_version: 1,
            chain_id: 0,
            best_block: None,
            latency_ms: 0,
            score: 100.0,
            last_message_time: 0,
            messages_sent: 0,
            messages_received: 0,
        }
    }

    pub fn is_validator(&self) -> bool {
        self.role == PeerRole::Validator
    }

    pub fn update_latency(&mut self, latency_ms: u64) {
        self.latency_ms = latency_ms;
        self.score = (self.score + (1000.0 / (latency_ms as f64 + 1.0))) / 2.0;
    }

    pub fn increment_sent(&mut self) {
        self.messages_sent += 1;
    }

    pub fn increment_received(&mut self) {
        self.messages_received += 1;
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl From<[u8; 32]> for PeerId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkAddress {
    pub protocol: TransportProtocol,
    pub host: String,
    pub port: u16,
}

impl NetworkAddress {
    pub fn new_ip(host: String, port: u16) -> Self {
        Self {
            protocol: TransportProtocol::Tcp,
            host,
            port,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.host, self.port, self.protocol.as_str())
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Quic,
    WebSocket,
}

impl TransportProtocol {
    pub fn as_str(&self) -> &str {
        match self {
            TransportProtocol::Tcp => "tcp",
            TransportProtocol::Udp => "udp",
            TransportProtocol::Quic => "quic",
            TransportProtocol::WebSocket => "ws",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerSet {
    peers: HashMap<PeerId, PeerInfo>,
    banned_peers: HashMap<PeerId, u64>,
}

impl PeerSet {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            banned_peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, peer: PeerInfo) {
        self.peers.insert(peer.peer_id, peer);
    }

    pub fn remove_peer(&mut self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }

    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    pub fn get_peer_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerInfo> {
        self.peers.get_mut(peer_id)
    }

    pub fn update_peer(&mut self, peer_info: PeerInfo) {
        self.peers.insert(peer_info.peer_id, peer_info);
    }

    pub fn ban_peer(&mut self, peer_id: PeerId, duration_blocks: u64) {
        self.banned_peers.insert(peer_id, duration_blocks);
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.state = ConnectionState::Banned;
        }
    }

    pub fn unban_peer(&mut self, peer_id: &PeerId) -> bool {
        if self.banned_peers.remove(peer_id).is_some() {
            if let Some(peer) = self.peers.get_mut(peer_id) {
                peer.state = ConnectionState::Connected;
            }
            true
        } else {
            false
        }
    }

    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        self.banned_peers.contains_key(peer_id)
    }

    pub fn validators(&self) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.role == PeerRole::Validator)
            .collect()
    }

    pub fn full_nodes(&self) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.role == PeerRole::FullNode)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
}

impl Default for PeerSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_type: NetworkMessageType,
    pub sender: PeerId,
    pub receiver: Option<PeerId>,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub request_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkMessageType {
    Handshake,
    BlockRequest,
    BlockResponse,
    BlockAnnounce,
    TransactionAnnounce,
    TransactionHashes,
    ConsensusMessage,
    VoteMessage,
    FinalitySignature,
    StateRequest,
    StateResponse,
    SnapshotRequest,
    SnapshotResponse,
    Ping,
    Pong,
    GetPeers,
    Peers,
    Unknown,
}

impl NetworkMessage {
    pub fn new_block_announce(block_hash: Hash32, block_number: u64, sender: PeerId) -> Self {
        let mut payload = Vec::new();
        payload.extend_from_slice(&block_number.to_le_bytes());
        payload.extend_from_slice(block_hash.as_bytes());

        Self {
            message_type: NetworkMessageType::BlockAnnounce,
            sender,
            receiver: None,
            payload,
            timestamp: 0,
            request_id: None,
        }
    }

    pub fn new_vote_message(vote: VoteMessage, sender: PeerId) -> Self {
        Self {
            message_type: NetworkMessageType::VoteMessage,
            sender,
            receiver: None,
            payload: serde_json::to_vec(&vote).unwrap_or_default(),
            timestamp: 0,
            request_id: None,
        }
    }

    pub fn new_finality_signature(sig: FinalitySignature, sender: PeerId) -> Self {
        Self {
            message_type: NetworkMessageType::FinalitySignature,
            sender,
            receiver: None,
            payload: serde_json::to_vec(&sig).unwrap_or_default(),
            timestamp: 0,
            request_id: None,
        }
    }

    pub fn new_proposal(proposal: Proposal, sender: PeerId) -> Self {
        Self {
            message_type: NetworkMessageType::ConsensusMessage,
            sender,
            receiver: None,
            payload: serde_json::to_vec(&proposal).unwrap_or_default(),
            timestamp: 0,
            request_id: None,
        }
    }
}

pub struct GossipProtocol {
    pub peer_set: PeerSet,
    message_cache: HashMap<Hash32, u64>,
    seen_messages: HashMap<PeerId, HashSet<Hash32>>,
    max_cache_size: usize,
    message_ttl_ms: u64,
}

impl GossipProtocol {
    pub fn new() -> Self {
        Self {
            peer_set: PeerSet::new(),
            message_cache: HashMap::new(),
            seen_messages: HashMap::new(),
            max_cache_size: 10000,
            message_ttl_ms: 300000,
        }
    }

    pub fn broadcast(&mut self, message: &NetworkMessage, exclude: Option<&PeerId>) -> Vec<PeerId> {
        let message_hash = Self::hash_message(message);

        if self.message_cache.contains_key(&message_hash) {
            return Vec::new();
        }

        self.message_cache.insert(message_hash, message.timestamp);
        if self.message_cache.len() > self.max_cache_size {
            self.prune_cache();
        }

        let mut targets = Vec::new();
        for (peer_id, peer_info) in &self.peer_set.peers {
            if peer_info.state == ConnectionState::Connected {
                if let Some(ex) = exclude {
                    if peer_id == ex {
                        continue;
                    }
                }
                targets.push(*peer_id);
            }
        }

        targets
    }

    pub fn send_to(&mut self, message: &NetworkMessage, receiver: &PeerId) -> bool {
        if let Some(peer) = self.peer_set.get_peer_mut(receiver) {
            if peer.state == ConnectionState::Connected {
                peer.increment_sent();
                return true;
            }
        }
        false
    }

    pub fn receive_message(&mut self, message: &NetworkMessage) -> bool {
        let message_hash = Self::hash_message(message);

        if self.message_cache.contains_key(&message_hash) {
            return false;
        }

        self.seen_messages
            .entry(message.sender)
            .or_insert_with(HashSet::new)
            .insert(message_hash);

        if let Some(peer) = self.peer_set.get_peer_mut(&message.sender) {
            peer.increment_received();
        }

        self.message_cache.insert(message_hash, message.timestamp);
        true
    }

    pub fn has_seen(&self, peer_id: &PeerId, message_hash: &Hash32) -> bool {
        self.seen_messages
            .get(peer_id)
            .map(|s| s.contains(message_hash))
            .unwrap_or(false)
    }

    pub fn add_peer(&mut self, peer_info: PeerInfo) {
        self.peer_set.add_peer(peer_info);
    }

    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peer_set.remove_peer(peer_id);
        self.seen_messages.remove(peer_id);
    }

    fn hash_message(message: &NetworkMessage) -> Hash32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&message.payload);
        hasher.update(&message.sender.0);
        hasher.update(&message.timestamp.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result[..32]);
        Hash32(hash)
    }

    fn prune_cache(&mut self) {
        let now: u64 = 0;
        let to_remove: Vec<Hash32> = self
            .message_cache
            .iter()
            .filter(|(_, &timestamp)| now.saturating_sub(timestamp) > self.message_ttl_ms)
            .map(|(hash, _)| *hash)
            .collect();

        for hash in to_remove {
            self.message_cache.remove(&hash);
        }
    }
}

impl Default for GossipProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockRequest {
    pub block_hashes: Vec<Hash32>,
    pub block_numbers: Vec<u64>,
    pub direction: BlockRequestDirection,
    pub max_blocks: u32,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockRequestDirection {
    Ascending,
    Descending,
}

impl BlockRequest {
    pub fn new_by_hash(hashes: Vec<Hash32>) -> Self {
        Self {
            block_hashes: hashes,
            block_numbers: Vec::new(),
            direction: BlockRequestDirection::Descending,
            max_blocks: 64,
        }
    }

    pub fn new_by_number(block_numbers: Vec<u64>) -> Self {
        Self {
            block_hashes: Vec::new(),
            block_numbers,
            direction: BlockRequestDirection::Ascending,
            max_blocks: 64,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    pub blocks: Vec<BlockData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockData {
    pub hash: Hash32,
    pub number: u64,
    pub header: Vec<u8>,
    pub body: Vec<Vec<u8>>,
    pub receipts: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionAnnounce {
    pub tx_hash: Hash32,
    pub sender: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockAnnounce {
    pub header: Vec<u8>,
    pub block_hash: Hash32,
    pub number: u64,
    pub is_new_best: bool,
}

impl BlockAnnounce {
    pub fn new(header: Vec<u8>, block_hash: Hash32, number: u64, is_new_best: bool) -> Self {
        Self {
            header,
            block_hash,
            number,
            is_new_best,
        }
    }
}

pub struct NetworkConfig {
    pub listen_addresses: Vec<NetworkAddress>,
    pub max_peers: u32,
    pub max_incoming_peers: u32,
    pub min_peers_for_sync: u32,
    pub ping_interval_ms: u64,
    pub connection_timeout_ms: u64,
    pub handshake_timeout_ms: u64,
    pub enable_tls: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addresses: Vec::new(),
            max_peers: 50,
            max_incoming_peers: 10,
            min_peers_for_sync: 3,
            ping_interval_ms: 30000,
            connection_timeout_ms: 10000,
            handshake_timeout_ms: 5000,
            enable_tls: false,
        }
    }
}

use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_info_creation() {
        let peer_id = PeerId::from_bytes([1u8; 32]);
        let addr = NetworkAddress::new_ip("127.0.0.1".to_string(), 30333);
        let peer = PeerInfo::new(peer_id, addr);

        assert!(!peer.is_validator());
        assert_eq!(peer.state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_peer_set_management() {
        let mut peer_set = PeerSet::new();

        let peer_id = PeerId::from_bytes([1u8; 32]);
        let addr = NetworkAddress::new_ip("127.0.0.1".to_string(), 30333);
        let peer = PeerInfo::new(peer_id, addr);

        peer_set.add_peer(peer);

        assert_eq!(peer_set.len(), 1);
        assert!(peer_set.get_peer(&peer_id).is_some());
    }

    #[test]
    fn test_gossip_broadcast() {
        let mut gossip = GossipProtocol::new();

        let peer_id = PeerId::from_bytes([1u8; 32]);
        let peer = PeerInfo::new(
            peer_id,
            NetworkAddress::new_ip("127.0.0.1".to_string(), 30333),
        );
        gossip.add_peer(peer);

        let message =
            NetworkMessage::new_block_announce(Hash32::empty(), 1, PeerId::from_bytes([2u8; 32]));

        let targets = gossip.broadcast(&message, None);
        assert!(targets.is_empty());
    }
}
