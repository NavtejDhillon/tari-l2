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
use tari_l2_p2p::P2PNetwork;
use crate::storage::MarketplaceStorage;
use crate::escrow::EscrowContract;
use tracing::info;

/// Manages all marketplace channels and operations
pub struct MarketplaceManager {
    /// Active channels indexed by channel ID
    channels: Arc<RwLock<HashMap<Hash, MarketplaceChannel>>>,

    /// Global marketplace listings (not tied to specific channels)
    global_listings: Arc<RwLock<Vec<Listing>>>,

    /// Global orders (tracking purchases across the marketplace)
    global_orders: Arc<RwLock<Vec<Order>>>,

    /// Escrow contracts indexed by escrow ID
    escrow_contracts: Arc<RwLock<HashMap<Hash, EscrowContract>>>,

    /// Persistent storage
    storage: Arc<MarketplaceStorage>,

    /// Node's keypair
    keypair: Arc<KeyPair>,

    /// P2P network for broadcasting listings
    network: Option<Arc<P2PNetwork>>,

    /// Optional L1 client for blockchain operations
    l1_client: Option<Arc<tari_l2_l1_client::TariL1Client>>,
}

impl MarketplaceManager {
    /// Create a new marketplace manager
    pub fn new(
        storage: Arc<MarketplaceStorage>,
        keypair: Arc<KeyPair>,
        l1_client: Option<Arc<tari_l2_l1_client::TariL1Client>>,
    ) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            global_listings: Arc::new(RwLock::new(Vec::new())),
            global_orders: Arc::new(RwLock::new(Vec::new())),
            escrow_contracts: Arc::new(RwLock::new(HashMap::new())),
            storage,
            keypair,
            network: None,
            l1_client,
        }
    }

    /// Set the P2P network for broadcasting listings
    pub fn set_network(&mut self, network: Arc<P2PNetwork>) {
        self.network = Some(network);
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
        let channel = MarketplaceChannel::new(config.clone());
        let channel_id = channel.channel_id;

        info!("Creating channel: {:?}", channel_id);

        // Calculate total collateral
        let total_collateral: u64 = config.initial_balances.values().map(|a| a.value()).sum();

        // Lock collateral on L1 if client available
        if let Some(ref l1_client) = self.l1_client {
            let participants: Vec<String> = config.participants
                .iter()
                .map(|pk| format!("{:?}", pk))
                .collect();

            match l1_client.lock_collateral(channel_id.to_string(), total_collateral, participants).await {
                Ok(tx_id) => {
                    info!("✅ Locked {} units of collateral on L1, tx: {}", total_collateral, tx_id);
                }
                Err(e) => {
                    info!("⚠️  Failed to lock collateral on L1: {}. Continuing without L1 lock.", e);
                }
            }
        }

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

        // Get final balances before closing
        let final_balances: HashMap<String, u64> = channel.participants
            .iter()
            .filter_map(|participant| {
                channel.get_balance(participant).ok().map(|balance| {
                    (format!("{:?}", participant), balance.value())
                })
            })
            .collect();

        channel.initiate_close()?;

        // Unlock collateral on L1 if client available
        if let Some(ref l1_client) = self.l1_client {
            match l1_client.unlock_collateral(channel_id.to_string(), final_balances.clone()).await {
                Ok(tx_id) => {
                    info!("✅ Unlocked collateral on L1, tx: {}", tx_id);
                }
                Err(e) => {
                    info!("⚠️  Failed to unlock collateral on L1: {}", e);
                }
            }
        }

        // Persist changes
        self.storage.store_channel(channel)?;

        info!("Closing channel: {:?}", channel_id);
        Ok(())
    }

    /// Get L1 connection status
    pub fn get_l1_status(&self) -> Option<String> {
        self.l1_client.as_ref().map(|client| {
            let status = client.get_status();
            format!("Network: {:?}, Endpoint: {}, Connected: {}",
                    status.network, status.endpoint, status.connected)
        })
    }

    /// Create a global marketplace listing
    pub async fn create_global_listing(
        &self,
        id: Hash,
        seller: PublicKey,
        title: String,
        description: String,
        price: u64,
        ipfs_hash: String,
    ) -> Result<()> {
        let listing = Listing {
            id,
            seller,
            title: title.clone(),
            description,
            price: Amount::new(price),
            ipfs_hash,
            active: true,
        };

        // Persist to database first
        self.storage.store_listing(&listing)?;

        // Add to in-memory cache
        self.global_listings.write().await.push(listing.clone());

        // Broadcast to P2P network
        if let Some(network) = &self.network {
            let listing_bytes = bincode::serialize(&listing)
                .map_err(|e| L2Error::SerializationError(e.to_string()))?;
            let signature = self.keypair.sign(&listing_bytes);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let message = tari_l2_p2p::L2Message::ListingBroadcast {
                listing,
                signature,
                timestamp,
            };

            network.broadcast_message(message).await
                .map_err(|e| L2Error::Unknown(format!("Failed to broadcast listing: {}", e)))?;

            info!("✅ Created and broadcast global listing: {} - {}", title, price);
        } else {
            info!("⚠️  Created global listing (no P2P network): {} - {}", title, price);
        }

        Ok(())
    }

    /// Handle incoming listing from P2P network
    pub async fn handle_received_listing(&self, listing: Listing, signature: tari_l2_common::Signature, timestamp: u64) -> Result<()> {
        // Verify signature
        let listing_bytes = bincode::serialize(&listing)
            .map_err(|e| L2Error::SerializationError(e.to_string()))?;

        if !listing.seller.verify(&listing_bytes, &signature) {
            return Err(L2Error::InvalidSignature);
        }

        // Check if we already have this listing
        let listings = self.global_listings.read().await;
        if listings.iter().any(|l| l.id == listing.id) {
            return Ok(()); // Already have it
        }
        drop(listings);

        // Persist to database
        self.storage.store_listing(&listing)?;

        // Add to in-memory cache
        self.global_listings.write().await.push(listing.clone());

        info!("📦 Received and stored listing from network: {} (ID: {:?})", listing.title, listing.id);

        Ok(())
    }

    /// Load all listings from storage
    pub async fn load_listings(&self) -> Result<()> {
        let listings = self.storage.load_all_listings()?;
        let mut global_listings = self.global_listings.write().await;
        *global_listings = listings;
        info!("✅ Loaded {} listings from storage", global_listings.len());
        Ok(())
    }

    /// List all global marketplace listings
    pub async fn list_all_listings(&self) -> Vec<(Hash, Listing)> {
        // Return global listings (channel ID is not meaningful for global listings,
        // so we use a zero hash as placeholder)
        let listings = self.global_listings.read().await;
        listings.iter()
            .filter(|l| l.active)
            .map(|l| (Hash::new([0u8; 32]), l.clone()))
            .collect()
    }

    /// Get listings for a specific channel
    pub async fn get_channel_listings(&self, channel_id: &Hash) -> Result<Vec<Listing>> {
        let channels = self.channels.read().await;
        let channel = channels.get(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        Ok(channel.state.listings.iter()
            .filter(|l| l.active)
            .cloned()
            .collect())
    }

    /// List all orders across all channels
    pub async fn list_all_orders(&self) -> Vec<(Hash, Order)> {
        let channels = self.channels.read().await;
        let mut all_orders = Vec::new();

        for (channel_id, channel) in channels.iter() {
            for order in &channel.state.orders {
                all_orders.push((*channel_id, order.clone()));
            }
        }

        all_orders
    }

    /// Get orders for a specific channel
    pub async fn get_channel_orders(&self, channel_id: &Hash) -> Result<Vec<Order>> {
        let channels = self.channels.read().await;
        let channel = channels.get(channel_id)
            .ok_or_else(|| L2Error::ChannelNotFound(channel_id.to_string()))?;

        Ok(channel.state.orders.clone())
    }

    // ===== Escrow Management =====

    /// Create a new escrow contract for a purchase
    pub async fn create_escrow(
        &self,
        listing_id: Hash,
        buyer: PublicKey,
        seller: PublicKey,
        amount: Amount,
        timeout_period: u64,
    ) -> Result<Hash> {
        let escrow = EscrowContract::new(listing_id, buyer, seller, amount, timeout_period);
        let escrow_id = escrow.id;

        self.escrow_contracts.write().await.insert(escrow_id, escrow);
        info!("Created escrow contract: {:?}", escrow_id);

        Ok(escrow_id)
    }

    /// Fund an escrow contract (buyer deposits funds to L1)
    pub async fn fund_escrow(&self, escrow_id: &Hash, l1_tx_id: String) -> Result<()> {
        let mut escrows = self.escrow_contracts.write().await;
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| L2Error::Unknown(format!("Escrow not found: {:?}", escrow_id)))?;

        escrow.fund(l1_tx_id).map_err(|e| L2Error::Unknown(e))?;
        info!("Funded escrow: {:?}", escrow_id);

        Ok(())
    }

    /// Mark order as shipped (seller confirms shipment)
    pub async fn ship_order(&self, escrow_id: &Hash, tracking_info: Option<String>) -> Result<()> {
        let mut escrows = self.escrow_contracts.write().await;
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| L2Error::Unknown(format!("Escrow not found: {:?}", escrow_id)))?;

        escrow.mark_shipped(tracking_info).map_err(|e| L2Error::Unknown(e))?;
        info!("Marked escrow as shipped: {:?}", escrow_id);

        Ok(())
    }

    /// Confirm delivery and release funds to seller (buyer confirms receipt)
    pub async fn confirm_delivery(&self, escrow_id: &Hash) -> Result<()> {
        let mut escrows = self.escrow_contracts.write().await;
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| L2Error::Unknown(format!("Escrow not found: {:?}", escrow_id)))?;

        escrow.confirm_receipt().map_err(|e| L2Error::Unknown(e))?;

        // TODO: Release funds to seller on L1 when L1 escrow methods are implemented
        info!("Confirmed delivery and released escrow: {:?}", escrow_id);
        Ok(())
    }

    /// Request refund (buyer initiates refund request)
    pub async fn request_refund(&self, escrow_id: &Hash, reason: String) -> Result<()> {
        let mut escrows = self.escrow_contracts.write().await;
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| L2Error::Unknown(format!("Escrow not found: {:?}", escrow_id)))?;

        escrow.request_refund(reason).map_err(|e| L2Error::Unknown(e))?;
        info!("Refund requested for escrow: {:?}", escrow_id);

        Ok(())
    }

    /// Approve refund (seller agrees to refund)
    pub async fn approve_refund(&self, escrow_id: &Hash) -> Result<()> {
        let mut escrows = self.escrow_contracts.write().await;
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| L2Error::Unknown(format!("Escrow not found: {:?}", escrow_id)))?;

        escrow.approve_refund().map_err(|e| L2Error::Unknown(e))?;

        // TODO: Refund to buyer on L1 when L1 escrow methods are implemented
        info!("Approved refund for escrow: {:?}", escrow_id);
        Ok(())
    }

    /// Raise dispute (either party can dispute)
    pub async fn raise_dispute(&self, escrow_id: &Hash, reason: String) -> Result<()> {
        let mut escrows = self.escrow_contracts.write().await;
        let escrow = escrows.get_mut(escrow_id)
            .ok_or_else(|| L2Error::Unknown(format!("Escrow not found: {:?}", escrow_id)))?;

        escrow.raise_dispute(reason).map_err(|e| L2Error::Unknown(e))?;
        info!("Dispute raised for escrow: {:?}", escrow_id);

        Ok(())
    }

    /// Get escrow contract details
    pub async fn get_escrow(&self, escrow_id: &Hash) -> Result<EscrowContract> {
        let escrows = self.escrow_contracts.read().await;
        escrows.get(escrow_id)
            .cloned()
            .ok_or_else(|| L2Error::Unknown(format!("Escrow not found: {:?}", escrow_id)))
    }

    /// List all escrow contracts
    pub async fn list_escrows(&self) -> Vec<EscrowContract> {
        self.escrow_contracts.read().await.values().cloned().collect()
    }

    /// Process timeouts for escrows (auto-release to seller)
    pub async fn process_escrow_timeouts(&self) -> Result<Vec<Hash>> {
        let mut escrows = self.escrow_contracts.write().await;
        let mut released = Vec::new();

        for (escrow_id, escrow) in escrows.iter_mut() {
            if escrow.is_timed_out() {
                if let Ok(_) = escrow.auto_release() {
                    released.push(*escrow_id);
                    info!("Auto-released timed out escrow: {:?}", escrow_id);
                }
            }
        }

        Ok(released)
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
        let manager = MarketplaceManager::new(storage, keypair.clone(), None);

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
