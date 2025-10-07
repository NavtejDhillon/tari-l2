use libp2p::{
    noise, tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
    swarm::SwarmEvent,
    gossipsub,
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use crate::behaviour::L2Behaviour;
use crate::messages::L2Message;

pub struct SwarmManager {
    pub swarm: Swarm<L2Behaviour>,
    message_tx: mpsc::UnboundedSender<L2Message>,
}

impl SwarmManager {
    pub fn new(
        _listen_addr: Multiaddr,
        message_tx: mpsc::UnboundedSender<L2Message>,
    ) -> anyhow::Result<Self> {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let _local_peer_id = PeerId::from(local_key.public());

        info!("🔑 Local peer id: {:?}", _local_peer_id);

        let swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| anyhow::anyhow!("Failed to configure TCP: {}", e))?
            .with_behaviour(|_| L2Behaviour::new(local_key.clone()).unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to create behaviour: {}", e))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        Ok(Self {
            swarm,
            message_tx,
        })
    }

    pub async fn start(&mut self, listen_addr: Multiaddr) -> anyhow::Result<()> {
        self.swarm.listen_on(listen_addr.clone())
            .map_err(|e| anyhow::anyhow!("Failed to listen: {}", e))?;
        info!("📡 Listening on {:?}", listen_addr);

        self.swarm.behaviour_mut().subscribe("tari-l2-state-updates")
            .map_err(|e| anyhow::anyhow!("Failed to subscribe: {}", e))?;
        self.swarm.behaviour_mut().subscribe("tari-l2-channel-announcements")
            .map_err(|e| anyhow::anyhow!("Failed to subscribe: {}", e))?;

        info!("✅ Subscribed to network topics");
        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<SwarmEvent<behaviour::L2BehaviourEvent>> {
        self.swarm.select_next_some().await.into()
    }

    pub fn handle_event(&mut self, event: SwarmEvent<behaviour::L2BehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(behaviour::L2BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message_id,
                message,
            })) => {
                debug!("📨 Received message from {:?}: {:?}", propagation_source, message_id);

                match bincode::deserialize::<L2Message>(&message.data) {
                    Ok(l2_message) => {
                        info!("✅ Deserialized message: {:?}", l2_message.message_type());
                        if let Err(e) = self.message_tx.send(l2_message) {
                            error!("Failed to forward message: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to deserialize message: {}", e);
                    }
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("🎧 Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                info!("🤝 Connection established with {:?} at {:?}", peer_id, endpoint);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!("👋 Connection closed with {:?}: {:?}", peer_id, cause);
            }
            SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {
                debug!("📥 Incoming connection {} from {} to {}", connection_id, send_back_addr, local_addr);
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                warn!("❌ Outgoing connection error to {:?}: {}", peer_id, error);
            }
            SwarmEvent::IncomingConnectionError { error, .. } => {
                warn!("❌ Incoming connection error: {}", error);
            }
            _ => {}
        }
    }

    pub fn publish_message(&mut self, topic: &str, message: L2Message) -> anyhow::Result<()> {
        let data = bincode::serialize(&message)?;
        debug!("📤 Publishing {} bytes to topic: {}", data.len(), topic);
        self.swarm.behaviour_mut().publish(topic, data)
            .map_err(|e| anyhow::anyhow!("Failed to publish: {}", e))?;
        Ok(())
    }

    pub fn dial(&mut self, addr: Multiaddr) -> anyhow::Result<()> {
        info!("☎️  Dialing peer at: {}", addr);
        self.swarm.dial(addr)
            .map_err(|e| anyhow::anyhow!("Failed to dial: {}", e))?;
        Ok(())
    }
}

use crate::behaviour;
