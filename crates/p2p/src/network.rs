use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error, debug, warn};
use tari_l2_common::{PublicKey, error::Result, error::L2Error};
use crate::messages::L2Message;
use crate::handler::MessageHandler;
use crate::swarm_manager::SwarmManager;

/// P2P network configuration
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfig {
    /// Listen address (libp2p multiaddr format)
    pub listen_addr: String,

    /// Bootstrap peers (libp2p multiaddr format)
    pub bootstrap_peers: Vec<String>,

    /// Maximum number of peers
    pub max_peers: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/9000".to_string(),
            bootstrap_peers: Vec::new(),
            max_peers: 50,
        }
    }
}

/// P2P network for L2 nodes
pub struct P2PNetwork {
    config: NetworkConfig,
    peers: Arc<RwLock<HashMap<PeerId, PublicKey>>>,
    message_tx: mpsc::UnboundedSender<L2Message>,
    message_rx: Arc<RwLock<mpsc::UnboundedReceiver<L2Message>>>,
    swarm_tx: Arc<RwLock<Option<mpsc::UnboundedSender<SwarmCommand>>>>,
}

enum SwarmCommand {
    Publish { topic: String, message: L2Message },
    Dial { addr: Multiaddr },
}

impl P2PNetwork {
    /// Create a new P2P network
    pub fn new(config: NetworkConfig) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_tx,
            message_rx: Arc::new(RwLock::new(message_rx)),
            swarm_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the P2P network
    pub async fn start(&self) -> Result<()> {
        info!("Starting P2P network on {}", self.config.listen_addr);

        let listen_addr = Multiaddr::from_str(&self.config.listen_addr)
            .map_err(|e| L2Error::Unknown(format!("Invalid listen address: {}", e)))?;

        // Create channels for swarm commands
        let (swarm_cmd_tx, mut swarm_cmd_rx) = mpsc::unbounded_channel();

        // Store the command sender
        *self.swarm_tx.write().await = Some(swarm_cmd_tx);

        // Create and start the swarm manager
        let message_tx = self.message_tx.clone();
        let bootstrap_peers = self.config.bootstrap_peers.clone();

        tokio::spawn(async move {
            match SwarmManager::new(listen_addr.clone(), message_tx) {
                Ok(mut swarm_manager) => {
                    // Start listening
                    if let Err(e) = swarm_manager.start(listen_addr).await {
                        error!("Failed to start swarm: {}", e);
                        return;
                    }

                    // Subscribe to marketplace topics
                    if let Err(e) = swarm_manager.swarm.behaviour_mut().subscribe("tari-l2-marketplace") {
                        error!("Failed to subscribe to marketplace topic: {}", e);
                    }

                    // Connect to bootstrap peers
                    for peer_addr in bootstrap_peers {
                        match Multiaddr::from_str(&peer_addr) {
                            Ok(addr) => {
                                info!("Connecting to bootstrap peer: {}", addr);
                                if let Err(e) = swarm_manager.dial(addr) {
                                    warn!("Failed to dial bootstrap peer: {}", e);
                                }
                            }
                            Err(e) => {
                                warn!("Invalid bootstrap peer address {}: {}", peer_addr, e);
                            }
                        }
                    }

                    // Run swarm event loop and handle commands
                    loop {
                        tokio::select! {
                            Some(event) = swarm_manager.next_event() => {
                                swarm_manager.handle_event(event);
                            }
                            Some(cmd) = swarm_cmd_rx.recv() => {
                                match cmd {
                                    SwarmCommand::Publish { topic, message } => {
                                        if let Err(e) = swarm_manager.publish_message(&topic, message) {
                                            error!("Failed to publish message: {}", e);
                                        }
                                    }
                                    SwarmCommand::Dial { addr } => {
                                        if let Err(e) = swarm_manager.dial(addr) {
                                            error!("Failed to dial peer: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create swarm manager: {}", e);
                }
            }
        });

        info!("P2P network started successfully");
        Ok(())
    }

    /// Broadcast a message to all connected peers
    pub async fn broadcast_message(&self, message: L2Message) -> Result<()> {
        let swarm_tx = self.swarm_tx.read().await;

        if let Some(tx) = swarm_tx.as_ref() {
            let topic = match &message {
                L2Message::ListingBroadcast { .. } => "tari-l2-marketplace",
                L2Message::StateUpdateProposal { .. } => "tari-l2-state-updates",
                L2Message::ChannelOpenRequest { .. } => "tari-l2-channel-announcements",
                _ => "tari-l2-general",
            };

            tx.send(SwarmCommand::Publish {
                topic: topic.to_string(),
                message,
            }).map_err(|e| L2Error::Unknown(format!("Failed to send publish command: {}", e)))?;

            debug!("Broadcast message to topic: {}", topic);
        } else {
            warn!("Swarm not initialized, cannot broadcast message");
        }

        Ok(())
    }

    /// Send a message to a specific peer (via gossipsub for now)
    pub async fn send_message(&self, _peer: PublicKey, message: L2Message) -> Result<()> {
        self.broadcast_message(message).await
    }

    /// Get list of connected peers
    pub async fn connected_peers(&self) -> Vec<PublicKey> {
        let peers = self.peers.read().await;
        peers.values().copied().collect()
    }

    /// Connect to a peer
    pub async fn connect_peer(&self, addr: &str) -> Result<()> {
        info!("Connecting to peer: {}", addr);

        let multiaddr = Multiaddr::from_str(addr)
            .map_err(|e| L2Error::Unknown(format!("Invalid peer address: {}", e)))?;

        let swarm_tx = self.swarm_tx.read().await;

        if let Some(tx) = swarm_tx.as_ref() {
            tx.send(SwarmCommand::Dial { addr: multiaddr })
                .map_err(|e| L2Error::Unknown(format!("Failed to send dial command: {}", e)))?;
        }

        Ok(())
    }

    /// Process messages with a handler
    pub async fn process_messages<H: MessageHandler + Send + Sync + 'static>(&self, handler: Arc<H>) -> Result<()> {
        let mut rx = self.message_rx.write().await;

        info!("Starting message processing loop");

        while let Some(message) = rx.recv().await {
            let handler = handler.clone();

            tokio::spawn(async move {
                debug!("Processing message: {:?}", message.message_type());

                let dummy_sender = PublicKey::new([0u8; 32]);

                match handler.handle_message(dummy_sender, message).await {
                    Ok(Some(response)) => {
                        debug!("Message handled, response: {:?}", response.message_type());
                    }
                    Ok(None) => {
                        debug!("Message handled successfully");
                    }
                    Err(e) => {
                        error!("Error handling message: {}", e);
                    }
                }
            });
        }

        Ok(())
    }
}
