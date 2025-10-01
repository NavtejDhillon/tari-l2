use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tari_l2_common::{Amount, Hash, PublicKey, crypto};

/// Channel state containing all marketplace data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelState {
    /// Nonce for ordering state updates
    pub nonce: u64,

    /// Participant balances
    pub balances: HashMap<PublicKey, Amount>,

    /// Marketplace listings (simplified for now)
    pub listings: Vec<Listing>,

    /// Active orders
    pub orders: Vec<Order>,
}

impl ChannelState {
    pub fn new(_participants: Vec<PublicKey>, initial_balances: HashMap<PublicKey, Amount>) -> Self {
        Self {
            nonce: 0,
            balances: initial_balances,
            listings: Vec::new(),
            orders: Vec::new(),
        }
    }

    /// Calculate merkle root of current state for L1 anchoring
    pub fn merkle_root(&self) -> Hash {
        let serialized = bincode::serialize(self).expect("Serialization should not fail");
        crypto::hash_data(&serialized)
    }

    /// Get balance for a participant
    pub fn get_balance(&self, participant: &PublicKey) -> Amount {
        self.balances.get(participant).copied().unwrap_or(Amount::ZERO)
    }

    /// Update balance for a participant
    pub fn set_balance(&mut self, participant: PublicKey, amount: Amount) {
        self.balances.insert(participant, amount);
    }

    /// Increment nonce
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Listing {
    pub id: Hash,
    pub seller: PublicKey,
    pub title: String,
    pub description: String,
    pub price: Amount,
    pub ipfs_hash: String,  // For images and additional data
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: Hash,
    pub listing_id: Hash,
    pub buyer: PublicKey,
    pub seller: PublicKey,
    pub amount: Amount,
    pub status: OrderStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipping,
    Delivered,
    Disputed,
    Completed,
    Cancelled,
}
