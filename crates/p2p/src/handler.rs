use async_trait::async_trait;
use tari_l2_common::{PublicKey, error::Result};
use crate::messages::L2Message;

/// Handler for incoming P2P messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message from a peer
    async fn handle_message(&self, from: PublicKey, message: L2Message) -> Result<Option<L2Message>>;
}
