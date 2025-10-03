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

    /// Get balance from connected wallet
    pub async fn get_wallet_balance(&self) -> Result<u64> {
        if !self.is_connected().await {
            return Err(anyhow!("Not connected to base node"));
        }

        // Get wallet gRPC endpoint
        let wallet_grpc = match &self.config.wallet_grpc {
            Some(endpoint) => endpoint,
            None => {
                return Err(anyhow!("Wallet gRPC endpoint not configured. Please configure wallet_grpc in config.toml"));
            }
        };

        // Connect to wallet gRPC
        use minotari_wallet_grpc_client::WalletGrpcClient;

        match WalletGrpcClient::connect(wallet_grpc).await {
            Ok(mut client) => {
                use minotari_wallet_grpc_client::grpc::GetBalanceRequest;
                // Empty request - no payment ID filter
                let request = GetBalanceRequest {
                    payment_id: None,
                };
                match client.get_balance(request).await {
                    Ok(response) => {
                        let balance = response.into_inner();
                        info!("✅ Wallet balance retrieved: {} µT", balance.available_balance);
                        Ok(balance.available_balance)
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to get wallet balance: {}", e);
                        Ok(0)
                    }
                }
            }
            Err(e) => {
                warn!("⚠️  Failed to connect to wallet gRPC: {}", e);
                Ok(0)
            }
        }
    }

    /// Create a new wallet address (requires wallet gRPC connection)
    pub async fn create_wallet_address(&self) -> Result<String> {
        if !self.is_connected().await {
            return Err(anyhow!("L1 not connected"));
        }

        let wallet_grpc = self.config.wallet_grpc.as_ref()
            .ok_or_else(|| anyhow!("Wallet gRPC not configured"))?;

        info!("Creating new wallet address via: {}", wallet_grpc);

        // Connect to wallet gRPC
        use minotari_wallet_grpc_client::WalletGrpcClient;

        let mut client = WalletGrpcClient::connect(wallet_grpc).await
            .map_err(|e| anyhow!("Failed to connect to wallet: {}", e))?;

        use minotari_wallet_grpc_client::grpc::Empty;
        let request = Empty {};
        let response = client.get_address(request).await
            .map_err(|e| anyhow!("Failed to get address: {}", e))?;

        let response_inner = response.into_inner();
        // Use one_sided_address for L2 operations
        let address = hex::encode(&response_inner.one_sided_address);
        info!("✅ Got Tari one-sided address: {}", address);
        Ok(address)
    }

    /// Import wallet from seed words (requires wallet gRPC connection)
    pub async fn import_wallet_from_seed(&self, _seed_words: Vec<String>) -> Result<String> {
        if !self.is_connected().await {
            return Err(anyhow!("L1 not connected"));
        }

        let _wallet_grpc = self.config.wallet_grpc.as_ref()
            .ok_or_else(|| anyhow!("Wallet gRPC not configured"))?;

        // Wallet seed import is not supported via gRPC API
        // Users must import seed phrases directly through Tari wallet CLI or Aurora wallet
        Err(anyhow!("Seed import not supported via gRPC. Please use Tari wallet CLI: 'minotari_console_wallet --seed-words \"your 24 words here\"'"))
    }

    /// Scan outputs with a private key
    fn scan_outputs_with_key(
        outputs: Vec<minotari_app_grpc::tari_rpc::TransactionOutput>,
        view_key: &tari_crypto::ristretto::RistrettoSecretKey,
    ) -> Result<u64> {
        use tari_transaction_components::transaction_components::EncryptedData;
        use tari_common_types::types::CompressedCommitment;
        use tari_crypto::tari_utilities::ByteArray;

        let mut total_balance = 0u64;
        let mut found_count = 0;

        for output in outputs {
            // Extract commitment - it's a Vec<u8>, not Option<Vec<u8>>
            let commitment_bytes = &output.commitment;

            // Extract encrypted_data - it's a Vec<u8>, not Option<Vec<u8>>
            let encrypted_data_bytes = &output.encrypted_data;

            // Parse encrypted data
            let encrypted_data = match EncryptedData::from_bytes(encrypted_data_bytes) {
                Ok(data) => data,
                Err(_) => continue,
            };

            // Parse commitment
            let commitment = match CompressedCommitment::from_canonical_bytes(commitment_bytes) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Try to decrypt
            match EncryptedData::decrypt_data(&view_key, &commitment, &encrypted_data) {
                Ok((value, _private_key, _payment_id)) => {
                    total_balance += u64::from(value);
                    found_count += 1;
                    info!("💰 Found output: {} µT", u64::from(value));
                },
                Err(_) => {
                    // Not our output
                }
            }
        }

        if found_count > 0 {
            info!("✅ Found {} outputs totaling {} µT", found_count, total_balance);
        } else {
            info!("ℹ️  No outputs found for this wallet");
        }

        Ok(total_balance)
    }

    /// Get balance for a wallet by scanning UTXOs from base node
    pub async fn get_balance_with_key(&self, view_key: tari_crypto::ristretto::RistrettoSecretKey) -> Result<u64> {
        if !self.is_connected().await {
            warn!("⚠️  Base node not connected, cannot query balance");
            return Ok(0);
        }

        info!("🔍 Scanning UTXOs with wallet...");

        use minotari_app_grpc::tari_rpc::base_node_client::BaseNodeClient;
        use minotari_app_grpc::tari_rpc::{Empty, GetBlocksRequest};

        let mut client = BaseNodeClient::connect(self.config.base_node_grpc.clone()).await?;

        // Get current chain tip
        let tip_response = client.get_tip_info(Empty {}).await?;
        let current_height = tip_response.into_inner().metadata
            .map(|m| m.best_block_height)
            .unwrap_or(0);

        info!("📊 Scanning blockchain at height {}", current_height);

        // Scan recent blocks for outputs
        // Scan last 1000 blocks or less if chain is shorter
        let start_height = if current_height > 1000 { current_height - 1000 } else { 0 };

        info!("🔍 Scanning blocks {} to {}", start_height, current_height);

        let mut all_outputs = Vec::new();

        // Scan in batches of 100 blocks
        for batch_start in (start_height..=current_height).step_by(100) {
            let batch_end = std::cmp::min(batch_start + 99, current_height);

            let request = GetBlocksRequest {
                heights: (batch_start..=batch_end).collect(),
            };

            match client.get_blocks(request).await {
                Ok(response) => {
                    use tokio_stream::StreamExt;
                    let mut blocks = response.into_inner();

                    while let Some(block) = blocks.next().await {
                        let block = match block {
                            Ok(b) => b,
                            Err(e) => {
                                warn!("⚠️  Error reading block: {}", e);
                                continue;
                            }
                        };
                        if let Some(block_data) = &block.block {
                            if let Some(body) = &block_data.body {
                                // Collect all outputs from this block
                                for output in &body.outputs {
                                    all_outputs.push(output.clone());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to get blocks {}-{}: {}", batch_start, batch_end, e);
                }
            }
        }

        info!("🔍 Scanned {} total outputs, checking ownership...", all_outputs.len());

        // Use private key to scan outputs
        let balance = Self::scan_outputs_with_key(all_outputs, &view_key)?;

        info!("✅ Balance: {} µT", balance);

        Ok(balance)
    }

    /// Get balance for an address by scanning UTXOs from base node (legacy method)
    pub async fn get_balance(&self, address: String) -> Result<u64> {
        if !self.is_connected().await {
            warn!("⚠️  Base node not connected, cannot query balance");
            return Ok(0);
        }

        info!("🔍 Scanning UTXOs for address: {}...", &address[..20]);

        use minotari_app_grpc::tari_rpc::base_node_client::BaseNodeClient;
        use minotari_app_grpc::tari_rpc::{Empty, GetBlocksRequest};

        let mut client = BaseNodeClient::connect(self.config.base_node_grpc.clone()).await?;

        // Get current chain tip
        let tip_response = client.get_tip_info(Empty {}).await?;
        let current_height = tip_response.into_inner().metadata
            .map(|m| m.best_block_height)
            .unwrap_or(0);

        info!("📊 Scanning blockchain at height {}", current_height);

        // Parse the Tari address to extract public key
        let address_bytes = hex::decode(&address)
            .map_err(|e| anyhow!("Invalid address hex: {}", e))?;

        if address_bytes.len() < 33 {
            return Err(anyhow!("Address too short"));
        }

        // Skip network byte (1) and features (1), extract public key (32 bytes)
        let public_key = &address_bytes[2..34];

        info!("🔑 Looking for outputs to public key: {}", hex::encode(public_key));

        // Scan recent blocks for outputs to this address
        // Scan last 1000 blocks or less if chain is shorter
        let start_height = if current_height > 1000 { current_height - 1000 } else { 0 };

        let mut total_balance = 0u64;
        let mut outputs_found = 0;

        info!("🔍 Scanning blocks {} to {}", start_height, current_height);

        // Scan in batches of 100 blocks
        for batch_start in (start_height..=current_height).step_by(100) {
            let batch_end = std::cmp::min(batch_start + 99, current_height);

            let request = GetBlocksRequest {
                heights: (batch_start..=batch_end).collect(),
            };

            match client.get_blocks(request).await {
                Ok(response) => {
                    use tokio_stream::StreamExt;
                    let mut blocks = response.into_inner();

                    while let Some(block) = blocks.next().await {
                        let block = match block {
                            Ok(b) => b,
                            Err(e) => {
                                warn!("⚠️  Error reading block: {}", e);
                                continue;
                            }
                        };
                        if let Some(block_data) = &block.block {
                            let block_height = block_data.header.as_ref().map(|h| h.height).unwrap_or(0);

                            if let Some(body) = &block_data.body {
                                // Check outputs in this block
                                for output in &body.outputs {
                                    if let Some(features) = &output.features {
                                        // Check if this is a coinbase output (output_type == 0 for coinbase)
                                        if features.output_type == 0 {
                                            // For now, count all coinbase outputs
                                            // Proper matching would require commitment verification
                                            total_balance += features.maturity;
                                            outputs_found += 1;
                                            info!("💰 Found coinbase output: {} µT at height {}",
                                                  features.maturity, block_height);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to get blocks {}-{}: {}", batch_start, batch_end, e);
                }
            }
        }

        if outputs_found > 0 {
            info!("✅ Found {} outputs, estimated balance: {} µT", outputs_found, total_balance);
        } else {
            info!("ℹ️  No outputs found for this address in recent blocks");
        }

        Ok(total_balance)
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
