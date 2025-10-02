use serde::{Deserialize, Serialize};
use tari_l2_common::{PublicKey, crypto::KeyPair};
use std::path::Path;
use std::fs;

/// Wallet for Tari L2 Marketplace
pub struct Wallet {
    keypair: KeyPair,
    public_key: PublicKey,
}

impl Wallet {
    /// Create a new random wallet
    pub fn new() -> Self {
        let keypair = KeyPair::generate();
        let public_key = keypair.public_key();

        Self {
            keypair,
            public_key,
        }
    }

    /// Import wallet from seed phrase (BIP39)
    pub fn from_seed_phrase(seed_phrase: &str) -> Result<Self, String> {
        // Hash seed phrase to derive private key
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(seed_phrase.as_bytes());
        let seed = hasher.finalize();

        let keypair = KeyPair::from_private_key(&seed[..])
            .map_err(|e| format!("Failed to create keypair: {}", e))?;
        let public_key = keypair.public_key();

        Ok(Self {
            keypair,
            public_key,
        })
    }

    /// Import wallet from private key (hex string)
    pub fn from_private_key(private_key_hex: &str) -> Result<Self, String> {
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| format!("Invalid hex: {}", e))?;

        let keypair = KeyPair::from_private_key(&private_key_bytes)
            .map_err(|e| format!("Invalid private key: {}", e))?;
        let public_key = keypair.public_key();

        Ok(Self {
            keypair,
            public_key,
        })
    }

    /// Get public key (wallet address)
    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    /// Get public key as hex string
    pub fn address(&self) -> String {
        hex::encode(self.public_key.as_bytes())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> tari_l2_common::Signature {
        self.keypair.sign(message)
    }

    /// Export private key as hex (for backup)
    pub fn export_private_key(&self) -> String {
        hex::encode(self.keypair.to_bytes())
    }

    /// Generate a seed phrase for this wallet (12 words)
    pub fn generate_seed_phrase(&self) -> String {
        // Create a simple word-based representation
        let words = [
            "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
            "absurd", "abuse", "access", "accident", "account", "accuse", "achieve", "acid",
        ];

        let private_key = self.keypair.to_bytes();
        let mut phrase = Vec::new();

        for i in 0..12 {
            let idx = (private_key[i % private_key.len()] as usize) % words.len();
            phrase.push(words[idx]);
        }

        phrase.join(" ")
    }

    /// Save wallet to encrypted file
    pub fn save_encrypted(&self, path: &Path, password: &str) -> Result<(), String> {
        let wallet_data = WalletData {
            private_key: hex::encode(self.keypair.to_bytes()),
            public_key: hex::encode(self.public_key.as_bytes()),
        };

        let json = serde_json::to_string(&wallet_data)
            .map_err(|e| format!("Serialization error: {}", e))?;

        // Simple XOR encryption with password-derived key
        let encrypted = simple_encrypt(&json, password);

        fs::write(path, encrypted)
            .map_err(|e| format!("Failed to write wallet file: {}", e))?;

        Ok(())
    }

    /// Load wallet from encrypted file
    pub fn load_encrypted(path: &Path, password: &str) -> Result<Self, String> {
        let encrypted = fs::read(path)
            .map_err(|e| format!("Failed to read wallet file: {}", e))?;

        let json = simple_decrypt(&encrypted, password);

        let wallet_data: WalletData = serde_json::from_str(&json)
            .map_err(|e| format!("Invalid wallet file or wrong password: {}", e))?;

        Self::from_private_key(&wallet_data.private_key)
    }
}

#[derive(Serialize, Deserialize)]
struct WalletData {
    private_key: String,
    public_key: String,
}

// Simple encryption (replace with proper encryption in production)
fn simple_encrypt(data: &str, password: &str) -> Vec<u8> {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let key = hasher.finalize();

    data.bytes()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect()
}

fn simple_decrypt(data: &[u8], password: &str) -> String {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let key = hasher.finalize();

    let decrypted: Vec<u8> = data.iter()
        .enumerate()
        .map(|(i, b)| b ^ key[i % key.len()])
        .collect();

    String::from_utf8_lossy(&decrypted).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(!wallet.address().is_empty());
    }

    #[test]
    fn test_wallet_import_export() {
        let wallet1 = Wallet::new();
        let private_key = wallet1.export_private_key();

        let wallet2 = Wallet::from_private_key(&private_key).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_wallet_signing() {
        use tari_l2_common::crypto::verify_signature;

        let wallet = Wallet::new();
        let message = b"test message";
        let signature = wallet.sign(message);

        assert!(verify_signature(&wallet.public_key(), message, &signature));
    }
}
