use serde::{Deserialize, Serialize};
use tari_l2_common::{Hash, PublicKey};
use tari_l2_state_channel::{
    update::SignedStateUpdate,
    channel::ChannelInfo,
};

/// L2 network message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum L2Message {
    /// Request to open a new channel
    ChannelOpenRequest {
        participants: Vec<PublicKey>,
        initiator: PublicKey,
    },

    /// Response to channel open request
    ChannelOpenResponse {
        channel_id: Hash,
        accepted: bool,
    },

    /// State update proposal
    StateUpdateProposal {
        channel_id: Hash,
        update: SignedStateUpdate,
    },

    /// State update acknowledgment
    StateUpdateAck {
        channel_id: Hash,
        nonce: u64,
        signature: tari_l2_common::Signature,
    },

    /// Request channel info
    ChannelInfoRequest {
        channel_id: Hash,
    },

    /// Channel info response
    ChannelInfoResponse {
        info: Option<ChannelInfo>,
    },

    /// Ping message for keepalive
    Ping,

    /// Pong response
    Pong,
}

impl L2Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            L2Message::ChannelOpenRequest { .. } => MessageType::ChannelOpenRequest,
            L2Message::ChannelOpenResponse { .. } => MessageType::ChannelOpenResponse,
            L2Message::StateUpdateProposal { .. } => MessageType::StateUpdateProposal,
            L2Message::StateUpdateAck { .. } => MessageType::StateUpdateAck,
            L2Message::ChannelInfoRequest { .. } => MessageType::ChannelInfoRequest,
            L2Message::ChannelInfoResponse { .. } => MessageType::ChannelInfoResponse,
            L2Message::Ping => MessageType::Ping,
            L2Message::Pong => MessageType::Pong,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageType {
    ChannelOpenRequest,
    ChannelOpenResponse,
    StateUpdateProposal,
    StateUpdateAck,
    ChannelInfoRequest,
    ChannelInfoResponse,
    Ping,
    Pong,
}
