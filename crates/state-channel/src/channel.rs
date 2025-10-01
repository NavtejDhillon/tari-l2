use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tari_l2_common::{Amount, Hash, PublicKey, crypto};
use crate::state::ChannelState;
use crate::update::SignedStateUpdate;
use tari_l2_common::{L2Error, error::Result};

/// Configuration for creating a channel
#[derive(Clone, Debug)]
pub struct ChannelConfig {
    pub participants: Vec<PublicKey>,
    pub initial_balances: HashMap<PublicKey, Amount>,
    pub challenge_period: u64,  // In seconds
}

/// Status of a marketplace channel
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChannelStatus {
    /// Channel is being established
    Opening,
    /// Channel is active and processing transactions
    Active,
    /// Channel is being cooperatively closed
    Closing,
    /// Channel closure is being challenged
    Challenged,
    /// Channel is closed
    Closed,
}

/// Marketplace state channel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketplaceChannel {
    /// Unique channel identifier
    pub channel_id: Hash,

    /// Participants in this channel
    pub participants: Vec<PublicKey>,

    /// Total collateral locked on L1
    pub collateral: Amount,

    /// Current channel state
    pub state: ChannelState,

    /// Channel status
    pub status: ChannelStatus,

    /// Challenge period in seconds
    pub challenge_period: u64,

    /// History of signed state updates (kept for auditing)
    pub state_history: Vec<SignedStateUpdate>,
}

impl MarketplaceChannel {
    /// Create a new channel
    pub fn new(config: ChannelConfig) -> Self {
        let channel_data = bincode::serialize(&config.participants)
            .expect("Serialization should not fail");
        let channel_id = crypto::hash_data(&channel_data);

        let collateral = config.initial_balances.values()
            .fold(Amount::ZERO, |acc, &amount| {
                acc.checked_add(amount).expect("Collateral overflow")
            });

        let state = ChannelState::new(config.participants.clone(), config.initial_balances);

        Self {
            channel_id,
            participants: config.participants,
            collateral,
            state,
            status: ChannelStatus::Opening,
            challenge_period: config.challenge_period,
            state_history: Vec::new(),
        }
    }

    /// Apply a signed state update
    pub fn apply_update(&mut self, signed_update: SignedStateUpdate) -> Result<()> {
        // Verify channel is active
        if self.status != ChannelStatus::Active {
            return Err(L2Error::InvalidChannelState);
        }

        // Verify nonce is correct (must be current nonce + 1)
        if signed_update.nonce != self.state.nonce + 1 {
            return Err(L2Error::InvalidStateTransition);
        }

        // Verify all signatures
        if !signed_update.verify(&self.participants) {
            return Err(L2Error::InvalidSignature);
        }

        // Apply the update
        let new_state = signed_update.update.apply(self.state.clone())?;

        // Update channel state
        self.state = new_state;
        self.state_history.push(signed_update);

        Ok(())
    }

    /// Get the latest state root for L1 checkpointing
    pub fn get_state_root(&self) -> Hash {
        self.state.merkle_root()
    }

    /// Mark channel as active
    pub fn activate(&mut self) -> Result<()> {
        if self.status != ChannelStatus::Opening {
            return Err(L2Error::InvalidChannelState);
        }
        self.status = ChannelStatus::Active;
        Ok(())
    }

    /// Initiate cooperative close
    pub fn initiate_close(&mut self) -> Result<()> {
        if self.status != ChannelStatus::Active {
            return Err(L2Error::InvalidChannelState);
        }
        self.status = ChannelStatus::Closing;
        Ok(())
    }

    /// Get participant balance
    pub fn get_balance(&self, participant: &PublicKey) -> Result<Amount> {
        if !self.participants.contains(participant) {
            return Err(L2Error::ParticipantNotFound);
        }
        Ok(self.state.get_balance(participant))
    }

    /// Get channel info summary
    pub fn info(&self) -> ChannelInfo {
        ChannelInfo {
            channel_id: self.channel_id,
            participants: self.participants.clone(),
            status: self.status.clone(),
            nonce: self.state.nonce,
            collateral: self.collateral,
            num_listings: self.state.listings.len(),
            num_orders: self.state.orders.len(),
        }
    }
}

/// Summary information about a channel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub channel_id: Hash,
    pub participants: Vec<PublicKey>,
    pub status: ChannelStatus,
    pub nonce: u64,
    pub collateral: Amount,
    pub num_listings: usize,
    pub num_orders: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tari_l2_common::crypto::KeyPair;

    #[test]
    fn test_channel_creation() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();

        let mut balances = HashMap::new();
        balances.insert(kp1.public_key(), Amount::new(1000));
        balances.insert(kp2.public_key(), Amount::new(1000));

        let config = ChannelConfig {
            participants: vec![kp1.public_key(), kp2.public_key()],
            initial_balances: balances,
            challenge_period: 3600,
        };

        let channel = MarketplaceChannel::new(config);
        assert_eq!(channel.status, ChannelStatus::Opening);
        assert_eq!(channel.collateral, Amount::new(2000));
        assert_eq!(channel.participants.len(), 2);
    }

    #[test]
    fn test_channel_activation() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();

        let mut balances = HashMap::new();
        balances.insert(kp1.public_key(), Amount::new(1000));
        balances.insert(kp2.public_key(), Amount::new(1000));

        let config = ChannelConfig {
            participants: vec![kp1.public_key(), kp2.public_key()],
            initial_balances: balances,
            challenge_period: 3600,
        };

        let mut channel = MarketplaceChannel::new(config);
        assert!(channel.activate().is_ok());
        assert_eq!(channel.status, ChannelStatus::Active);
    }
}
