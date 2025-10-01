use libp2p::{
    noise, tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
    swarm::SwarmEvent,
    gossipsub,
};
use futures::StreamExt;
use std::error::Error;
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use crate::behaviour::L2Behaviour;
use crate::messages::L2Message;

pub struct SwarmManager {
    swarm: Swarm<L2Behaviour>,
    message_tx: mpsc::UnboundedSender<L2Message>,
}

impl SwarmManager {
    pub fn new(
        _listen_addr: Multiaddr,
        message_tx: mpsc::UnboundedSender<L2Message>,
    ) -> Result<Self, Box<dyn Error>> {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Local peer id: {:?}", local_peer_id);

        let swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| L2Behaviour::new(local_key.clone()).unwrap())?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        Ok(Self {
            swarm,
            message_tx,
        })
    }

    pub async fn start(&mut self, listen_addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on(listen_addr.clone())?;
        info!("Listening on {:?}", listen_addr);

        // Subscribe to default topics
        self.swarm.behaviour_mut().subscribe("tari-l2-state-updates")?;
        self.swarm.behaviour_mut().subscribe("tari-l2-channel-announcements")?;

        info!("Subscribed to network topics");
        Ok(())
    }

    pub async fn run(&mut self) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(event) => {
                    if let behaviour::L2BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source,
                        message_id,
                        message,
                    }) = event {
                        debug!("Received message from {:?}: {:?}", propagation_source, message_id);

                        // Deserialize and forward message
                        match bincode::deserialize::<L2Message>(&message.data) {
                            Ok(l2_message) => {
                                if let Err(e) = self.message_tx.send(l2_message) {
                                    error!("Failed to forward message: {}", e);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to deserialize message: {}", e);
                            }
                        }
                    }
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {}", address);
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    info!("Connection established with {:?} at {:?}", peer_id, endpoint);
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    info!("Connection closed with {:?}: {:?}", peer_id, cause);
                }
                _ => {}
            }
        }
    }

    pub fn publish_message(&mut self, topic: &str, message: L2Message) -> Result<(), Box<dyn Error>> {
        let data = bincode::serialize(&message)?;
        self.swarm.behaviour_mut().publish(topic, data)?;
        Ok(())
    }

    pub fn dial(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error>> {
        self.swarm.dial(addr)?;
        Ok(())
    }
}

// Re-export the event type for pattern matching
use crate::behaviour;
