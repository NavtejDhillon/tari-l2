use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tari_l2_common::{Amount, Hash, PublicKey, L2Error, error::Result, crypto::KeyPair};
use tari_l2_state_channel::{
    MarketplaceChannel, ChannelConfig, StateUpdate,
    channel::ChannelInfo,
    update::SignedStateUpdate,
    state::{Listing, Order, OrderStatus},
};
use crate::storage::MarketplaceStorage;
use tracing::info;

/// Manages all marketplace channels and operations
pub struct MarketplaceManager {
    /// Active channels indexed by channel ID
    channels: Arc<RwLock<HashMap<Hash, MarketplaceChannel>>>,

    /// Persistent storage
    storage: Arc<MarketplaceStorage>,

    /// Node's keypair
    keypair: Arc<KeyPair>,
}

impl MarketplaceManager {
    /// Create a new marketplace manager
    pub fn new(storage: Arc<MarketplaceStorage>, keypair: Arc<KeyPair>) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            storage,
            keypair,
        }
    }

    /// Load all channels from storage
    pub async fn load_channels(&self) -> Result<()> {
        let channel_ids = self.storage.list_channels()?;
        let mut channels = self.channels.write().await;

        for channel_id in channel_ids {
            if let Some(channel) = self.storage.load_channel(&channel_id)? {
                info!("Loaded channel: {:?}", channel_id);
                channels.insert(channel_id, channel);
            }
        }

        Ok(())
    }

    /// Create a new channel
    pub async fn create_channel(&self, config: ChannelConfig) -> Result<Hash> {
        let channel = MarketplaceChannel::new(config);
        let channel_id = channel.channel_id;

        info!("Creating channel: {:?}", channel_id);

        // Store in memory
        let mut channels = self.channels.write().await;
        if channels.contains_key(&channel_id) {
            return Err(L2Error::ChannelAlreadyExists(channel_id.to_string()));
        }
        channels.insert(channel_id, channel.clone());
        drop(channels);

        // Persist to storage
        self.storage.store_channel(&channel)?;

        Ok(channel_id)
    }

    /// Activate a channel
    pub async fn activate_channel(&self, channel_id: &Hash) -> Result<()> {
        let mut channels = self.channels.write().await;

        let channel = channels.get_mut(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        channel.activate()?;

        // Persist changes
        self.storage.store_channel(channel)?;

        info!("Activated channel: {:?}", channel_id);
        Ok(())
    }

    /// Get channel info
    pub async fn get_channel_info(&self, channel_id: &Hash) -> Result<ChannelInfo> {
        let channels = self.channels.read().await;
        let channel = channels.get(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        Ok(channel.info())
    }

    /// Create and sign a state update
    pub async fn create_state_update(
        &self,
        channel_id: &Hash,
        update: StateUpdate,
    ) -> Result<SignedStateUpdate> {
        let channels = self.channels.read().await;
        let channel = channels.get(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        let nonce = channel.state.nonce + 1;
        let mut signed_update = SignedStateUpdate::new(update, nonce);

        // Sign the update
        let message = bincode::serialize(&signed_update.update)
            .map_err(|e| L2Error::SerializationError(e.to_string()))?;
        let mut signing_data = message;
        signing_data.extend_from_slice(&nonce.to_le_bytes());

        let signature = self.keypair.sign(&signing_data);
        signed_update.add_signature(self.keypair.public_key(), signature);

        Ok(signed_update)
    }

    /// Add a signature to a pending state update
    pub async fn add_signature_to_update(
        &self,
        signed_update: &mut SignedStateUpdate,
    ) -> Result<()> {
        let message = bincode::serialize(&signed_update.update)
            .map_err(|e| L2Error::SerializationError(e.to_string()))?;
        let mut signing_data = message;
        signing_data.extend_from_slice(&signed_update.nonce.to_le_bytes());

        let signature = self.keypair.sign(&signing_data);
        signed_update.add_signature(self.keypair.public_key(), signature);

        Ok(())
    }

    /// Apply a signed state update to a channel
    pub async fn apply_state_update(
        &self,
        channel_id: &Hash,
        signed_update: SignedStateUpdate,
    ) -> Result<()> {
        let mut channels = self.channels.write().await;

        let channel = channels.get_mut(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        channel.apply_update(signed_update)?;

        // Persist changes
        self.storage.store_channel(channel)?;

        info!("Applied state update to channel: {:?}", channel_id);
        Ok(())
    }

    /// Create a new listing
    pub async fn create_listing(
        &self,
        channel_id: &Hash,
        listing: Listing,
    ) -> Result<SignedStateUpdate> {
        let update = StateUpdate::CreateListing { listing };
        self.create_state_update(channel_id, update).await
    }

    /// Create a new order
    pub async fn create_order(
        &self,
        channel_id: &Hash,
        order: Order,
    ) -> Result<SignedStateUpdate> {
        let update = StateUpdate::CreateOrder { order };
        self.create_state_update(channel_id, update).await
    }

    /// Update order status
    pub async fn update_order_status(
        &self,
        channel_id: &Hash,
        order_id: Hash,
        status: OrderStatus,
    ) -> Result<SignedStateUpdate> {
        let update = StateUpdate::UpdateOrderStatus { order_id, status };
        self.create_state_update(channel_id, update).await
    }

    /// Transfer funds between participants
    pub async fn transfer(
        &self,
        channel_id: &Hash,
        from: PublicKey,
        to: PublicKey,
        amount: Amount,
    ) -> Result<SignedStateUpdate> {
        let update = StateUpdate::Transfer { from, to, amount };
        self.create_state_update(channel_id, update).await
    }

    /// Get balance for a participant
    pub async fn get_balance(&self, channel_id: &Hash, participant: &PublicKey) -> Result<Amount> {
        let channels = self.channels.read().await;
        let channel = channels.get(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        channel.get_balance(participant)
    }

    /// List all channels
    pub async fn list_channels(&self) -> Vec<ChannelInfo> {
        let channels = self.channels.read().await;
        channels.values().map(|c| c.info()).collect()
    }

    /// Close a channel
    pub async fn close_channel(&self, channel_id: &Hash) -> Result<()> {
        let mut channels = self.channels.write().await;

        let channel = channels.get_mut(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        channel.initiate_close()?;

        // Persist changes
        self.storage.store_channel(channel)?;

        info!("Closing channel: {:?}", channel_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_marketplace_manager() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(MarketplaceStorage::open(temp_dir.path()).unwrap());
        let keypair = Arc::new(KeyPair::generate());
        let manager = MarketplaceManager::new(storage, keypair.clone());

        let kp2 = KeyPair::generate();

        let mut balances = HashMap::new();
        balances.insert(keypair.public_key(), Amount::new(1000));
        balances.insert(kp2.public_key(), Amount::new(1000));

        let config = ChannelConfig {
            participants: vec![keypair.public_key(), kp2.public_key()],
            initial_balances: balances,
            challenge_period: 3600,
        };

        // Create channel
        let channel_id = manager.create_channel(config).await.unwrap();

        // Activate channel
        manager.activate_channel(&channel_id).await.unwrap();

        // Get channel info
        let info = manager.get_channel_info(&channel_id).await.unwrap();
        assert_eq!(info.channel_id, channel_id);

        // Check balance
        let balance = manager.get_balance(&channel_id, &keypair.public_key()).await.unwrap();
        assert_eq!(balance, Amount::new(1000));
    }
}
