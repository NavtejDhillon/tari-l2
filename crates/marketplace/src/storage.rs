use sled::{Db, Tree};
use tari_l2_common::{Hash, L2Error, error::Result};
use tari_l2_state_channel::MarketplaceChannel;
use std::path::Path;

/// Persistent storage for marketplace state
pub struct MarketplaceStorage {
    _db: Db,
    channels: Tree,
}

impl MarketplaceStorage {
    /// Open or create a new database
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| L2Error::DatabaseError(e.to_string()))?;

        let channels = db.open_tree("channels")
            .map_err(|e| L2Error::DatabaseError(e.to_string()))?;

        Ok(Self { _db: db, channels })
    }

    /// Store a channel
    pub fn store_channel(&self, channel: &MarketplaceChannel) -> Result<()> {
        let key = channel.channel_id.to_vec();
        let value = bincode::serialize(channel)
            .map_err(|e| L2Error::SerializationError(e.to_string()))?;

        self.channels.insert(key, value)
            .map_err(|e| L2Error::DatabaseError(e.to_string()))?;

        self.channels.flush()
            .map_err(|e| L2Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Load a channel by ID
    pub fn load_channel(&self, channel_id: &Hash) -> Result<Option<MarketplaceChannel>> {
        let key = channel_id.to_vec();

        match self.channels.get(key)
            .map_err(|e| L2Error::DatabaseError(e.to_string()))? {
            Some(value) => {
                let channel = bincode::deserialize(&value)
                    .map_err(|e| L2Error::SerializationError(e.to_string()))?;
                Ok(Some(channel))
            }
            None => Ok(None),
        }
    }

    /// List all channel IDs
    pub fn list_channels(&self) -> Result<Vec<Hash>> {
        let mut channels = Vec::new();

        for result in self.channels.iter() {
            let (key, _) = result.map_err(|e| L2Error::DatabaseError(e.to_string()))?;
            let channel_id = Hash::from_slice(&key)
                .map_err(|e| L2Error::SerializationError(e.to_string()))?;
            channels.push(channel_id);
        }

        Ok(channels)
    }

    /// Delete a channel
    pub fn delete_channel(&self, channel_id: &Hash) -> Result<()> {
        let key = channel_id.to_vec();
        self.channels.remove(key)
            .map_err(|e| L2Error::DatabaseError(e.to_string()))?;

        self.channels.flush()
            .map_err(|e| L2Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get total number of channels
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tari_l2_state_channel::ChannelConfig;
    use tari_l2_common::{Amount, crypto::KeyPair};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_storage_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = MarketplaceStorage::open(temp_dir.path()).unwrap();

        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();

        let mut balances = HashMap::new();
        balances.insert(kp1.public_key(), Amount::new(1000));
        balances.insert(kp2.public_key(), Amount::new(1000));

        let config = ChannelConfig {
            participants: vec![kp1.public_key(), kp2.public_key()],
            initial_balances: balances,
            challenge_period: 3600,
        };

        let channel = MarketplaceChannel::new(config);
        let channel_id = channel.channel_id;

        // Store channel
        storage.store_channel(&channel).unwrap();

        // Load channel
        let loaded = storage.load_channel(&channel_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().channel_id, channel_id);

        // List channels
        let channels = storage.list_channels().unwrap();
        assert_eq!(channels.len(), 1);

        // Delete channel
        storage.delete_channel(&channel_id).unwrap();
        let deleted = storage.load_channel(&channel_id).unwrap();
        assert!(deleted.is_none());
    }
}
