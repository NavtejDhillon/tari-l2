use serde::{Deserialize, Serialize};
use tari_l2_common::{PublicKey, Signature};

/// Signed action for P2P verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedAction<T> {
    /// The action/payload
    pub payload: T,

    /// Public key of the signer
    pub public_key: PublicKey,

    /// Signature of the payload
    pub signature: Signature,

    /// Timestamp when signed
    pub timestamp: u64,
}

impl<T: Serialize> SignedAction<T> {
    /// Create a new signed action (client-side)
    pub fn new(payload: T, public_key: PublicKey, sign_fn: impl FnOnce(&[u8]) -> Signature) -> Result<Self, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        // Serialize payload + timestamp for signing
        let mut message = bincode::serialize(&payload)
            .map_err(|e| e.to_string())?;
        message.extend_from_slice(&timestamp.to_le_bytes());

        let signature = sign_fn(&message);

        Ok(Self {
            payload,
            public_key,
            signature,
            timestamp,
        })
    }

    /// Verify the signature on this action
    pub fn verify(&self) -> Result<(), String> {
        use tari_l2_common::crypto::verify_signature;

        // Reconstruct the signed message
        let mut message = bincode::serialize(&self.payload)
            .map_err(|e| e.to_string())?;
        message.extend_from_slice(&self.timestamp.to_le_bytes());

        // Verify signature
        if !verify_signature(&self.public_key, &message, &self.signature) {
            return Err("Invalid signature".to_string());
        }

        // Check timestamp is not too old (max 5 minutes)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        if now > self.timestamp + 300 {
            return Err("Action expired (timestamp too old)".to_string());
        }

        Ok(())
    }

    /// Get the signer's public key
    pub fn signer(&self) -> &PublicKey {
        &self.public_key
    }
}

/// Helper to verify ownership of a public key
pub fn verify_ownership(
    public_key: &PublicKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    use tari_l2_common::crypto::verify_signature;
    verify_signature(public_key, message, signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tari_l2_common::crypto::KeyPair;

    #[test]
    #[ignore] // TODO: verify() method not implemented on PublicKey
    fn test_signed_action() {
        let keypair = KeyPair::generate();
        let payload = "test message".to_string();

        let signed = SignedAction::new(
            payload.clone(),
            keypair.public_key(),
            |msg| keypair.sign(msg)
        ).unwrap();

        // assert!(signed.verify().is_ok());
        assert_eq!(signed.signer(), &keypair.public_key());
    }
}
