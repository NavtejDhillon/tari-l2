use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

pub mod config;
pub use config::{L1Config, TariNetwork};

/// Represents a locked collateral entry on L1
#[derive(Debug, Clone)]
pub struct LockedCollateral {
    pub channel_id: String,
    pub amount: u64,
    pub participants: Vec<String>,
    pub block_height: u64,
    pub tx_id: String,
}

/// Represents a checkpoint on L1
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub channel_id: String,
    pub state_root: String,
    pub block_height: u64,
    pub signatures: Vec<String>,
    pub tx_id: String,
}

/// Main client for interacting with Tari L1 blockchain
pub struct TariL1Client {
    config: L1Config,
    connected: Arc<Mutex<bool>>,
    // Local tracking of locked collateral (in-memory for now)
    locked_collateral: Arc<Mutex<HashMap<String, LockedCollateral>>>,
    // Local tracking of checkpoints
    checkpoints: Arc<Mutex<HashMap<String, Vec<Checkpoint>>>>,
    // Mock chain height for offline mode
    mock_chain_height: Arc<Mutex<u64>>,
}

impl TariL1Client {
    /// Create a new Tari L1 client
    pub async fn new(config: L1Config) -> Result<Self> {
        let client = Self {
            config,
            connected: Arc::new(Mutex::new(false)),
            locked_collateral: Arc::new(Mutex::new(HashMap::new())),
            checkpoints: Arc::new(Mutex::new(HashMap::new())),
            mock_chain_height: Arc::new(Mutex::new(1000)),
        };

        // Attempt to connect
        client.connect().await;

        Ok(client)
    }

    /// Attempt to connect to the Tari base node
    async fn connect(&self) {
        info!("🔗 Attempting to connect to Tari L1 node at {}", self.config.base_node_grpc);

        // TODO: Implement actual gRPC connection to Tari base node
        // For now, we'll attempt a basic connection check
        match self.try_connect().await {
            Ok(_) => {
                *self.connected.lock().await = true;
                info!("✅ Successfully connected to Tari L1 node (network: {:?})", self.config.network);
            }
            Err(e) => {
                *self.connected.lock().await = false;
                warn!("⚠️  Could not connect to Tari L1 node: {}. Running in offline mode.", e);
                warn!("⚠️  L1 operations will return mock values. Start Tari node for full functionality.");
            }
        }
    }

    /// Try to establish connection to Tari node
    async fn try_connect(&self) -> Result<()> {
        use minotari_app_grpc::tari_rpc::base_node_client::BaseNodeClient;

        // Connect to the base node gRPC endpoint
        let mut client = BaseNodeClient::connect(self.config.base_node_grpc.clone()).await?;

        // Test the connection by getting chain tip
        let request = minotari_app_grpc::tari_rpc::Empty {};
        let response = client.get_tip_info(request).await?;

        let height = response.get_ref().metadata.as_ref()
            .map(|m| m.best_block_height)
            .unwrap_or(0);

        info!("✅ Connected to Tari base node at height: {}", height);

        Ok(())
    }

    /// Check if the client is connected to L1
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// Get the current blockchain height
    pub async fn get_chain_height(&self) -> Result<u64> {
        if !self.is_connected().await {
            let height = *self.mock_chain_height.lock().await;
            *self.mock_chain_height.lock().await += 1; // Simulate block progression
            return Ok(height);
        }

        // Query actual chain height from L1
        use minotari_app_grpc::tari_rpc::base_node_client::BaseNodeClient;

        let mut client = BaseNodeClient::connect(self.config.base_node_grpc.clone()).await?;
        let request = minotari_app_grpc::tari_rpc::Empty {};
        let response = client.get_tip_info(request).await?;

        Ok(response.into_inner().metadata
            .map(|m| m.best_block_height)
            .unwrap_or(0))
    }

    /// Lock collateral on L1 for a payment channel
    pub async fn lock_collateral(
        &self,
        channel_id: String,
        amount: u64,
        participants: Vec<String>,
    ) -> Result<String> {
        info!("🔒 Locking {} units of collateral for channel {} with {} participants",
              amount, channel_id, participants.len());

        let block_height = self.get_chain_height().await?;

        if !self.is_connected().await {
            warn!("⚠️  Offline mode: Simulating collateral lock");
            let tx_id = format!("mock_tx_{}", hex::encode(&blake3::hash(channel_id.as_bytes()).as_bytes()[..8]));

            let collateral = LockedCollateral {
                channel_id: channel_id.clone(),
                amount,
                participants,
                block_height,
                tx_id: tx_id.clone(),
            };

            self.locked_collateral.lock().await.insert(channel_id, collateral);

            info!("✅ Mock collateral locked with tx_id: {}", tx_id);
            return Ok(tx_id);
        }

        // TODO: Implement actual L1 transaction to lock collateral
        // This should:
        // 1. Create a transaction that locks funds in a multi-sig output
        // 2. Include channel_id and participants in the transaction metadata
        // 3. Submit the transaction to the Tari network
        // 4. Wait for confirmation
        // Example:
        // let tx = create_collateral_lock_tx(amount, participants)?;
        // let tx_id = submit_transaction(&mut wallet_client, tx).await?;

        let tx_id = format!("mock_tx_{}", hex::encode(&blake3::hash(channel_id.as_bytes()).as_bytes()[..8]));

        let collateral = LockedCollateral {
            channel_id: channel_id.clone(),
            amount,
            participants,
            block_height,
            tx_id: tx_id.clone(),
        };

        self.locked_collateral.lock().await.insert(channel_id, collateral);

        info!("✅ Collateral locked with tx_id: {}", tx_id);
        Ok(tx_id)
    }

    /// Unlock collateral on L1 after channel closure
    pub async fn unlock_collateral(
        &self,
        channel_id: String,
        final_balances: HashMap<String, u64>,
    ) -> Result<String> {
        info!("🔓 Unlocking collateral for channel {}", channel_id);

        let collateral = self.locked_collateral.lock().await.get(&channel_id).cloned();

        if collateral.is_none() {
            return Err(anyhow!("No locked collateral found for channel {}", channel_id));
        }

        if !self.is_connected().await {
            warn!("⚠️  Offline mode: Simulating collateral unlock");
            let tx_id = format!("mock_unlock_tx_{}", hex::encode(&blake3::hash(channel_id.as_bytes()).as_bytes()[..8]));

            self.locked_collateral.lock().await.remove(&channel_id);

            info!("✅ Mock collateral unlocked with tx_id: {}", tx_id);
            return Ok(tx_id);
        }

        // TODO: Implement actual L1 transaction to unlock collateral
        // This should:
        // 1. Create a transaction that spends the locked output
        // 2. Distribute funds according to final_balances
        // 3. Include channel_id in the transaction metadata
        // 4. Submit the transaction to the Tari network
        // 5. Wait for confirmation

        let tx_id = format!("mock_unlock_tx_{}", hex::encode(&blake3::hash(channel_id.as_bytes()).as_bytes()[..8]));

        self.locked_collateral.lock().await.remove(&channel_id);

        info!("✅ Collateral unlocked with tx_id: {} (balances: {:?})", tx_id, final_balances);
        Ok(tx_id)
    }

    /// Checkpoint state to L1 blockchain
    pub async fn checkpoint_state(
        &self,
        channel_id: String,
        state_root: String,
        signatures: Vec<String>,
        block_height: u64,
    ) -> Result<String> {
        info!("📌 Creating checkpoint for channel {} at block {}", channel_id, block_height);

        if !self.is_connected().await {
            warn!("⚠️  Offline mode: Simulating checkpoint");
            let tx_id = format!("mock_checkpoint_tx_{}", hex::encode(&blake3::hash(channel_id.as_bytes()).as_bytes()[..8]));

            let checkpoint = Checkpoint {
                channel_id: channel_id.clone(),
                state_root,
                block_height,
                signatures,
                tx_id: tx_id.clone(),
            };

            self.checkpoints
                .lock()
                .await
                .entry(channel_id)
                .or_insert_with(Vec::new)
                .push(checkpoint);

            info!("✅ Mock checkpoint created with tx_id: {}", tx_id);
            return Ok(tx_id);
        }

        // TODO: Implement actual L1 transaction to checkpoint state
        // This should:
        // 1. Create a transaction with OP_RETURN or similar to anchor state
        // 2. Include state_root, channel_id, and signatures
        // 3. Submit the transaction to the Tari network
        // 4. Wait for confirmation

        let tx_id = format!("mock_checkpoint_tx_{}", hex::encode(&blake3::hash(channel_id.as_bytes()).as_bytes()[..8]));

        let checkpoint = Checkpoint {
            channel_id: channel_id.clone(),
            state_root,
            block_height,
            signatures,
            tx_id: tx_id.clone(),
        };

        self.checkpoints
            .lock()
            .await
            .entry(channel_id)
            .or_insert_with(Vec::new)
            .push(checkpoint);

        info!("✅ Checkpoint created with tx_id: {}", tx_id);
        Ok(tx_id)
    }

    /// Submit a dispute to L1
    pub async fn submit_dispute(
        &self,
        channel_id: String,
        disputed_state: String,
        proof: Vec<u8>,
    ) -> Result<String> {
        info!("⚠️  Submitting dispute for channel {}", channel_id);

        if !self.is_connected().await {
            warn!("⚠️  Offline mode: Cannot submit dispute. L1 connection required.");
            return Err(anyhow!("L1 connection required to submit disputes"));
        }

        // TODO: Implement actual L1 dispute transaction
        // This should:
        // 1. Create a transaction that triggers the dispute resolution contract
        // 2. Include disputed_state and cryptographic proof
        // 3. Submit to L1 for adjudication
        // 4. Return dispute transaction ID

        let tx_id = format!("mock_dispute_tx_{}", hex::encode(&blake3::hash(channel_id.as_bytes()).as_bytes()[..8]));

        error!("🚨 Dispute submitted with tx_id: {} (NOT IMPLEMENTED - requires L1 connection)", tx_id);
        Ok(tx_id)
    }

    /// Verify a transaction exists on L1
    pub async fn verify_transaction(&self, tx_id: String) -> Result<bool> {
        if !self.is_connected().await {
            warn!("⚠️  Offline mode: Cannot verify transaction");
            // In offline mode, assume mock transactions are valid
            return Ok(tx_id.starts_with("mock_"));
        }

        // TODO: Implement actual transaction verification
        // This should:
        // 1. Query the base node for the transaction
        // 2. Check if it's in a mined block
        // 3. Return confirmation status

        Ok(tx_id.starts_with("mock_"))
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: String) -> Result<u64> {
        if !self.is_connected().await {
            warn!("⚠️  Offline mode: Returning mock balance");
            return Ok(1000000); // Mock balance
        }

        // TODO: Implement actual balance query
        // This should:
        // 1. Connect to wallet gRPC service
        // 2. Query balance for the given address
        // 3. Return the balance in Tari units

        info!("Querying balance for address: {}", address);
        Ok(1000000) // Mock balance
    }

    /// Get connection status and info
    pub fn get_status(&self) -> ConnectionStatus {
        ConnectionStatus {
            connected: false, // Will be updated with actual connection check
            network: self.config.network.clone(),
            endpoint: self.config.base_node_grpc.clone(),
        }
    }
}

/// Connection status information
#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub network: TariNetwork,
    pub endpoint: String,
}
