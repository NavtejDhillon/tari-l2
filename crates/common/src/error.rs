use thiserror::Error;

#[derive(Error, Debug)]
pub enum L2Error {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid state transition")]
    InvalidStateTransition,

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Channel already exists: {0}")]
    ChannelAlreadyExists(String),

    #[error("Invalid channel state")]
    InvalidChannelState,

    #[error("Participant not found")]
    ParticipantNotFound,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Tari connection error: {0}")]
    TariConnectionError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Timeout")]
    Timeout,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, L2Error>;
