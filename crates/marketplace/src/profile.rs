use serde::{Deserialize, Serialize};
use tari_l2_common::PublicKey;

/// User profile information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    /// Wallet public key (unique identifier)
    pub public_key: PublicKey,

    /// Display name
    pub name: String,

    /// Location (city, country, etc.)
    pub location: Option<String>,

    /// Profile bio/description
    pub bio: Option<String>,

    /// Contact email (optional)
    pub email: Option<String>,

    /// Profile avatar URL or hash
    pub avatar: Option<String>,

    /// Rating (0-5 stars)
    pub rating: f32,

    /// Number of completed transactions
    pub transactions_completed: u32,

    /// Timestamp when profile was created
    pub created_at: u64,
}

impl UserProfile {
    /// Create a new user profile
    pub fn new(public_key: PublicKey, name: String) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            public_key,
            name,
            location: None,
            bio: None,
            email: None,
            avatar: None,
            rating: 0.0,
            transactions_completed: 0,
            created_at,
        }
    }

    /// Update profile information
    pub fn update(&mut self, name: Option<String>, location: Option<String>, bio: Option<String>, email: Option<String>) {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(l) = location {
            self.location = Some(l);
        }
        if let Some(b) = bio {
            self.bio = Some(b);
        }
        if let Some(e) = email {
            self.email = Some(e);
        }
    }

    /// Get public key as hex string
    pub fn address(&self) -> String {
        hex::encode(self.public_key.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tari_l2_common::crypto::KeyPair;

    #[test]
    fn test_profile_creation() {
        let keypair = KeyPair::generate();
        let profile = UserProfile::new(keypair.public_key(), "Alice".to_string());

        assert_eq!(profile.name, "Alice");
        assert_eq!(profile.rating, 0.0);
        assert_eq!(profile.transactions_completed, 0);
    }

    #[test]
    fn test_profile_update() {
        let keypair = KeyPair::generate();
        let mut profile = UserProfile::new(keypair.public_key(), "Alice".to_string());

        profile.update(
            Some("Alice Smith".to_string()),
            Some("New York, USA".to_string()),
            Some("Crypto enthusiast".to_string()),
            None,
        );

        assert_eq!(profile.name, "Alice Smith");
        assert_eq!(profile.location, Some("New York, USA".to_string()));
        assert_eq!(profile.bio, Some("Crypto enthusiast".to_string()));
    }
}
