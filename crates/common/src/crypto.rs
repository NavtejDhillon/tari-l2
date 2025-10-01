use crate::types::{Hash, PublicKey, Signature};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey};

/// Keypair for signing operations
pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        Self { signing_key }
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        let verifying_key = self.signing_key.verifying_key();
        PublicKey::new(verifying_key.to_bytes())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        let sig = self.signing_key.sign(message);
        Signature::new(sig.to_bytes())
    }
}

/// Verify a signature
pub fn verify_signature(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
    let verifying_key = match VerifyingKey::from_bytes(public_key.as_bytes()) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let sig = match ed25519_dalek::Signature::from_slice(signature.as_bytes()) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    verifying_key.verify(message, &sig).is_ok()
}

/// Hash arbitrary data using BLAKE3
pub fn hash_data(data: &[u8]) -> Hash {
    let hash = blake3::hash(data);
    Hash::new(*hash.as_bytes())
}

/// Hash multiple pieces of data together
pub fn hash_multiple(data: &[&[u8]]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    for item in data {
        hasher.update(item);
    }
    Hash::new(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        let pk = kp.public_key();
        assert_eq!(pk.as_bytes().len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = KeyPair::generate();
        let message = b"test message";
        let signature = kp.sign(message);
        let public_key = kp.public_key();

        assert!(verify_signature(&public_key, message, &signature));
        assert!(!verify_signature(&public_key, b"wrong message", &signature));
    }

    #[test]
    fn test_hash_data() {
        let data = b"test data";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);
        assert_eq!(hash1, hash2);

        let different_data = b"different data";
        let hash3 = hash_data(different_data);
        assert_ne!(hash1, hash3);
    }
}
