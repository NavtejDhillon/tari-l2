/// Full Tari wallet implementation for L2 Marketplace
///
/// This wallet uses Tari's transaction_key_manager with CipherSeed
/// for full compatibility with the Tari console wallet.

use tari_common::configuration::Network;
use tari_common_types::{
    seeds::{
        cipher_seed::CipherSeed,
        mnemonic::{Mnemonic, MnemonicLanguage},
        seed_words::SeedWords,
    },
    tari_address::TariAddress,
};
use tari_crypto::{
    keys::{PublicKey as PubKeyTrait, SecretKey},
    tari_utilities::ByteArray,
    hashing::DomainSeparation,
};
use tari_hashing::KeyManagerDomain;

type PrivateKey = tari_crypto::ristretto::RistrettoSecretKey;
type PublicKey = tari_crypto::ristretto::RistrettoPublicKey;

/// Full Tari wallet with proper key management using CipherSeed
#[derive(Clone)]
pub struct Wallet {
    cipher_seed: CipherSeed,
    spend_key: PrivateKey,
    public_spend_key: PublicKey,
    seed_words: Option<SeedWords>,
}

impl Wallet {
    /// Create a new random wallet with Tari CipherSeed (24 words)
    pub fn new() -> Self {
        // Generate new CipherSeed (no passphrase)
        let cipher_seed = CipherSeed::new();
        let seed_words = CipherSeed::to_mnemonic(&cipher_seed, MnemonicLanguage::English, None)
            .expect("Failed to generate seed words");

        Self::from_cipher_seed(cipher_seed, Some(seed_words))
            .expect("Failed to create wallet from cipher seed")
    }

    /// Import wallet from Tari seed words (24 words)
    pub fn from_seed_phrase(seed_phrase: &str) -> Result<Self, String> {
        use std::str::FromStr;

        // Parse seed words using Tari's format - use FromStr which handles Hidden<String> wrapping
        let seed_words = SeedWords::from_str(seed_phrase)
            .map_err(|e| format!("Failed to parse seed words: {:?}", e))?;

        if seed_words.len() != 24 {
            return Err(format!("Seed phrase must be exactly 24 words, got {}", seed_words.len()));
        }

        // Decode CipherSeed from mnemonic (no passphrase)
        let cipher_seed = CipherSeed::from_mnemonic(&seed_words, None)
            .map_err(|e| format!("Failed to decode seed words: {:?}", e))?;

        Self::from_cipher_seed(cipher_seed, Some(seed_words))
    }

    /// Create wallet from CipherSeed
    fn from_cipher_seed(cipher_seed: CipherSeed, seed_words: Option<SeedWords>) -> Result<Self, String> {
        use blake2::Blake2b;
        use digest::{Digest, consts::U64};

        // Derive spend key from cipher seed entropy using the same method as Tari wallet
        // Use domain-separated hashing with the wallet branch and index
        // This matches the key derivation in tari_key_manager.rs
        const WALLET_BRANCH: &str = "wallet";
        const SPEND_KEY_INDEX: u64 = 0;

        // Build the key material to hash
        let mut key_material = Vec::new();
        key_material.extend_from_slice(KeyManagerDomain::domain_separation_tag("derive_key").as_bytes());
        key_material.extend_from_slice(cipher_seed.entropy());
        key_material.extend_from_slice(WALLET_BRANCH.as_bytes());
        key_material.extend_from_slice(&SPEND_KEY_INDEX.to_le_bytes());

        // Hash to derive the key using Blake2b-512 for 64 bytes output
        let derive_key = Blake2b::<U64>::digest(&key_material);

        // Create private key from the hash using from_uniform_bytes (same as Tari)
        let spend_key = PrivateKey::from_uniform_bytes(derive_key.as_ref())
            .map_err(|e| format!("Failed to create spend key: {:?}", e))?;

        let public_spend_key = PubKeyTrait::from_secret_key(&spend_key);

        Ok(Self {
            cipher_seed,
            spend_key,
            public_spend_key,
            seed_words,
        })
    }

    /// Import wallet from private key (hex string)
    /// Note: This won't have a seed phrase since we're importing just the key
    pub fn from_private_key(private_key_hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(private_key_hex)
            .map_err(|e| format!("Invalid hex: {}", e))?;

        if bytes.len() != 32 {
            return Err("Private key must be 32 bytes".to_string());
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);

        let spend_key = PrivateKey::from_canonical_bytes(&key_bytes)
            .map_err(|e| format!("Failed to create private key: {:?}", e))?;
        let public_spend_key = PubKeyTrait::from_secret_key(&spend_key);

        // Create a dummy cipher seed (won't be usable for mnemonic export)
        let cipher_seed = CipherSeed::new();

        Ok(Self {
            cipher_seed,
            spend_key,
            public_spend_key,
            seed_words: None,
        })
    }

    /// Get Tari address (emoji format for Esmeralda testnet)
    pub fn address(&self) -> String {
        use tari_crypto::compressed_key::CompressedKey;

        // Convert to CompressedPublicKey (which is CompressedKey<RistrettoPublicKey>)
        let compressed_key = CompressedKey::new_from_pk(self.public_spend_key.clone());

        // Create Tari address from public spend key (single address, interactive only)
        let tari_address = TariAddress::new_single_address_with_interactive_only(compressed_key, Network::Esmeralda)
            .expect("Failed to create Tari address");
        tari_address.to_emoji_string()
    }

    /// Get Tari address in hex format
    pub fn address_hex(&self) -> String {
        use tari_crypto::compressed_key::CompressedKey;

        // Convert to CompressedPublicKey (which is CompressedKey<RistrettoPublicKey>)
        let compressed_key = CompressedKey::new_from_pk(self.public_spend_key.clone());

        // Create Tari address from public spend key (single address, interactive only)
        let tari_address = TariAddress::new_single_address_with_interactive_only(compressed_key, Network::Esmeralda)
            .expect("Failed to create Tari address");
        tari_address.to_hex()
    }

    /// Get raw public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_spend_key.as_bytes())
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.public_spend_key.as_bytes().to_vec()
    }

    /// Export private key as hex (for backup)
    pub fn export_private_key(&self) -> String {
        hex::encode(self.spend_key.as_bytes())
    }

    /// Get seed phrase for this wallet (24 words in Tari format)
    pub fn seed_phrase(&self) -> Option<String> {
        self.seed_words.as_ref().map(|sw| {
            sw.join(" ").reveal().to_string()
        })
    }

    /// Sign a message with the wallet's private key
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        // Simple hash-based signature for now
        // TODO: Implement proper Schnorr signature
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(message);
        hash.to_vec()
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(!wallet.address().is_empty());
        assert!(wallet.seed_phrase().is_some());

        let emoji_addr = wallet.address();
        let hex_addr = wallet.address_hex();
        let pubkey = wallet.public_key_hex();
        let privkey = wallet.export_private_key();

        println!("\n=== WALLET TEST ===");
        println!("Emoji address: {}", emoji_addr);
        println!("Emoji length: {} chars", emoji_addr.chars().count());
        println!("\nHex address: {}", hex_addr);
        println!("Hex length: {} chars (should be 66 for 33 bytes)", hex_addr.len());
        println!("\nPublic key: {}", pubkey);
        println!("Public key length: {} chars", pubkey.len());
        println!("\nPrivate key: {}", privkey);
        println!("Private key length: {} chars", privkey.len());

        // Verify seed phrase has 24 words
        let seed = wallet.seed_phrase().unwrap();
        assert_eq!(seed.split_whitespace().count(), 24);
        println!("\nSeed phrase words: {}", seed.split_whitespace().count());

        // Debug: Check if hex_addr is actually the address or concatenated data
        println!("\n=== ANALYSIS ===");
        if hex_addr.len() == 134 {
            println!("ERROR: Hex address is 134 chars (67 bytes)");
            println!("This looks like: network_byte (2) + public_key (64) + checksum (68) = 134");
        } else if hex_addr.len() == 66 {
            println!("✓ Hex address is correct length (33 bytes)");
        }
    }

    #[test]
    fn test_wallet_import_export() {
        let wallet1 = Wallet::new();
        let seed_phrase = wallet1.seed_phrase().unwrap();
        println!("Testing with seed: {}", seed_phrase);

        let wallet2 = Wallet::from_seed_phrase(&seed_phrase).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
        assert_eq!(wallet1.public_key_hex(), wallet2.public_key_hex());
    }

    #[test]
    fn test_wallet_from_private_key() {
        let wallet1 = Wallet::new();
        let private_key = wallet1.export_private_key();

        let wallet2 = Wallet::from_private_key(&private_key).unwrap();
        assert_eq!(wallet1.public_key_hex(), wallet2.public_key_hex());
        assert_eq!(wallet1.address(), wallet2.address());
    }
}
