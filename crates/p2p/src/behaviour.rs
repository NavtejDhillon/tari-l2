use libp2p::{
    gossipsub::{self, IdentTopic, MessageId, ValidationMode},
    identify,
    swarm::NetworkBehaviour,
    PeerId,
};
use std::time::Duration;

/// Network behavior combining gossipsub and identify
#[derive(NetworkBehaviour)]
pub struct L2Behaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
}

impl L2Behaviour {
    pub fn new(local_key: libp2p::identity::Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        let local_peer_id = PeerId::from(local_key.public());

        // Configure gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(|message: &gossipsub::Message| {
                MessageId::from(&blake3::hash(&message.data).as_bytes()[..])
            })
            .build()
            .map_err(|e| format!("Failed to build gossipsub config: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| format!("Failed to create gossipsub behaviour: {}", e))?;

        // Configure identify
        let identify = identify::Behaviour::new(
            identify::Config::new("/tari-l2/1.0.0".to_string(), local_key.public())
        );

        Ok(Self {
            gossipsub,
            identify,
        })
    }

    pub fn subscribe(&mut self, topic: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let topic = IdentTopic::new(topic);
        self.gossipsub.subscribe(&topic)
            .map_err(|e| format!("Failed to subscribe to topic: {}", e).into())
    }

    pub fn publish(&mut self, topic: &str, data: Vec<u8>) -> Result<MessageId, Box<dyn std::error::Error>> {
        let topic = IdentTopic::new(topic);
        self.gossipsub.publish(topic, data)
            .map_err(|e| format!("Failed to publish message: {}", e).into())
    }
}
