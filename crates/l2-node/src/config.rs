use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tari_l2_p2p::NetworkConfig;

/// Configuration for the L2 node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Path to data directory
    pub data_dir: PathBuf,

    /// Tari node connection
    pub tari_node: TariNodeConfig,

    /// P2P network configuration
    pub network: NetworkConfig,

    /// RPC server configuration
    pub rpc: RpcConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TariNodeConfig {
    /// Tari node address
    pub address: String,

    /// Tari node port
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcConfig {
    /// RPC listen address
    pub listen_addr: String,

    /// RPC listen port
    pub port: u16,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            tari_node: TariNodeConfig {
                address: "127.0.0.1".to_string(),
                port: 18142,
            },
            network: NetworkConfig::default(),
            rpc: RpcConfig {
                listen_addr: "127.0.0.1".to_string(),
                port: 18000,
            },
        }
    }
}

impl NodeConfig {
    /// Load config from file (supports both JSON and TOML)
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;

        // Try TOML first, then JSON
        if path.ends_with(".toml") {
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(serde_json::from_str(&contents)?)
        }
    }

    /// Save config to file (JSON by default)
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if path.ends_with(".toml") {
            let contents = toml::to_string_pretty(self)?;
            std::fs::write(path, contents)?;
        } else {
            let contents = serde_json::to_string_pretty(self)?;
            std::fs::write(path, contents)?;
        }
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure data directory can be created
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}
