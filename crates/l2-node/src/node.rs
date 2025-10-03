use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};
use tari_l2_common::{crypto::KeyPair, error::Result};
use tari_l2_marketplace::{MarketplaceManager, MarketplaceStorage};
use tari_l2_p2p::{P2PNetwork, MessageHandler};
use tari_l2_rpc::{RpcApi, RpcServer};
use crate::config::NodeConfig;
use crate::tari_client::TariClient;
use tari_l2_l1_client::{TariL1Client, L1Config, TariNetwork};
use async_trait::async_trait;
use tari_l2_common::{PublicKey, L2Error};
use tari_l2_p2p::L2Message;

/// Main L2 node
pub struct L2Node {
    config: NodeConfig,
    keypair: Arc<KeyPair>,
    marketplace: Arc<MarketplaceManager>,
    network: Arc<P2PNetwork>,
    tari_client: Arc<TariClient>,
    l1_client: Arc<TariL1Client>,
}

impl L2Node {
    /// Create a new L2 node
    pub async fn new(config: NodeConfig) -> Result<Self> {
        info!("Initializing L2 node");

        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| L2Error::Unknown(format!("Failed to create data directory: {}", e)))?;

        // Initialize components
        let keypair = Arc::new(KeyPair::generate());
        info!("Node public key: {}", keypair.public_key());

        // Initialize storage
        let storage = Arc::new(
            MarketplaceStorage::open(&config.data_dir)
                .map_err(|e| L2Error::DatabaseError(e.to_string()))?
        );

        // Initialize L1 client
        let l1_config = L1Config {
            base_node_grpc: "http://127.0.0.1:18142".to_string(),
            wallet_grpc: None,
            network: TariNetwork::Esmeralda,
        };
        let l1_client = Arc::new(
            TariL1Client::new(l1_config).await
                .map_err(|e| L2Error::Unknown(format!("Failed to create L1 client: {}", e)))?
        );

        // Check L1 connection and log status
        if l1_client.is_connected().await {
            info!("✅ Connected to Tari L1 blockchain");
        } else {
            info!("⚠️  Running in offline mode - L1 blockchain not available");
        }

        // Initialize marketplace manager with L1 client
        let marketplace = Arc::new(MarketplaceManager::new(storage, keypair.clone(), Some(l1_client.clone())));

        // Load existing channels
        marketplace.load_channels().await?;

        // Load existing listings from storage
        marketplace.load_listings().await?;

        // Initialize P2P network
        let network = Arc::new(P2PNetwork::new(config.network.clone()));

        // Initialize Tari client
        let tari_client = Arc::new(TariClient::new(
            config.tari_node.address.clone(),
            config.tari_node.port,
        ));

        Ok(Self {
            config,
            keypair,
            marketplace,
            network,
            tari_client,
            l1_client,
        })
    }

    /// Start the L2 node
    pub async fn start(&self) -> Result<()> {
        info!("Starting L2 node");

        // Connect to Tari L1
        self.tari_client.connect().await?;

        // Start P2P network
        self.network.start().await?;

        // Start message processing
        let network = self.network.clone();
        let handler = Arc::new(NodeMessageHandler {
            marketplace: self.marketplace.clone(),
        });
        tokio::spawn(async move {
            if let Err(e) = network.process_messages(handler).await {
                error!("Error processing messages: {}", e);
            }
        });

        // Start RPC server
        let rpc_addr = format!("{}:{}", self.config.rpc.listen_addr, self.config.rpc.port)
            .parse()
            .map_err(|e| L2Error::InvalidParameter(format!("Invalid RPC address: {}", e)))?;

        let l1_connected = Arc::new(std::sync::atomic::AtomicBool::new(self.l1_client.is_connected().await));
        let api = Arc::new(RpcApi::new_with_l1(self.marketplace.clone(), self.l1_client.clone(), l1_connected));
        let rpc_server = RpcServer::new(api, rpc_addr);

        tokio::spawn(async move {
            if let Err(e) = rpc_server.start().await {
                error!("RPC server error: {}", e);
            }
        });

        info!("L2 node started successfully");
        info!("RPC server listening on {}", rpc_addr);

        // Wait for shutdown signal
        self.wait_for_shutdown().await;

        Ok(())
    }

    async fn wait_for_shutdown(&self) {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutdown signal received");
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.keypair.public_key()
    }
}

/// Message handler for L2 network messages
struct NodeMessageHandler {
    marketplace: Arc<MarketplaceManager>,
}

#[async_trait]
impl MessageHandler for NodeMessageHandler {
    async fn handle_message(
        &self,
        from: PublicKey,
        message: L2Message,
    ) -> Result<Option<L2Message>> {
        info!("Handling message from {:?}: {:?}", from, message.message_type());

        match message {
            L2Message::Ping => {
                Ok(Some(L2Message::Pong))
            }
            L2Message::StateUpdateProposal { channel_id, update } => {
                // Apply the state update
                match self.marketplace.apply_state_update(&channel_id, update).await {
                    Ok(()) => {
                        info!("Applied state update for channel {:?}", channel_id);
                        Ok(None)
                    }
                    Err(e) => {
                        error!("Failed to apply state update: {}", e);
                        Err(e)
                    }
                }
            }
            L2Message::ChannelInfoRequest { channel_id } => {
                match self.marketplace.get_channel_info(&channel_id).await {
                    Ok(info) => {
                        Ok(Some(L2Message::ChannelInfoResponse { info: Some(info) }))
                    }
                    Err(_) => {
                        Ok(Some(L2Message::ChannelInfoResponse { info: None }))
                    }
                }
            }
            L2Message::ListingBroadcast { listing, signature, timestamp } => {
                info!("📦 Received listing broadcast: {} from {:?}", listing.title, from);
                match self.marketplace.handle_received_listing(listing, signature, timestamp).await {
                    Ok(()) => {
                        info!("✅ Successfully processed listing from P2P network");
                        Ok(None)
                    }
                    Err(e) => {
                        error!("❌ Failed to process listing: {}", e);
                        Err(e)
                    }
                }
            }
            L2Message::ListingsRequest => {
                let listings = self.marketplace.list_all_listings().await;
                let listings_only: Vec<_> = listings.into_iter().map(|(_, listing)| listing).collect();
                Ok(Some(L2Message::ListingsResponse { listings: listings_only }))
            }
            L2Message::ListingsResponse { listings } => {
                info!("📦 Received {} listings from peer", listings.len());
                for listing in listings {
                    // Process each listing - using dummy signature/timestamp since these are responses
                    let dummy_sig = tari_l2_common::Signature::new([0u8; 64]);
                    if let Err(e) = self.marketplace.handle_received_listing(listing, dummy_sig, 0).await {
                        error!("Failed to process listing: {}", e);
                    }
                }
                Ok(None)
            }
            _ => {
                // Other message types not yet implemented
                Ok(None)
            }
        }
    }
}
