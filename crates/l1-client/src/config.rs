use serde::{Deserialize, Serialize};

/// Tari network types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TariNetwork {
    /// Mainnet
    Mainnet,
    /// Esmeralda testnet
    Esmeralda,
    /// Nextnet (experimental)
    Nextnet,
    /// Local development network
    Localnet,
}

impl Default for TariNetwork {
    fn default() -> Self {
        TariNetwork::Esmeralda
    }
}

/// Configuration for Tari L1 client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1Config {
    /// Base node gRPC endpoint
    pub base_node_grpc: String,
    /// Optional wallet gRPC endpoint
    pub wallet_grpc: Option<String>,
    /// Network type
    pub network: TariNetwork,
}

impl Default for L1Config {
    fn default() -> Self {
        Self::testnet()
    }
}

impl L1Config {
    /// Create config for Esmeralda testnet (default)
    pub fn testnet() -> Self {
        Self {
            base_node_grpc: "http://127.0.0.1:18143".to_string(),
            wallet_grpc: Some("http://127.0.0.1:18143".to_string()),
            network: TariNetwork::Esmeralda,
        }
    }

    /// Create config for mainnet
    pub fn mainnet() -> Self {
        Self {
            base_node_grpc: "http://127.0.0.1:18142".to_string(),
            wallet_grpc: Some("http://127.0.0.1:18142".to_string()),
            network: TariNetwork::Mainnet,
        }
    }

    /// Create config for localnet (development)
    pub fn localnet() -> Self {
        Self {
            base_node_grpc: "http://127.0.0.1:18142".to_string(),
            wallet_grpc: Some("http://127.0.0.1:18142".to_string()),
            network: TariNetwork::Localnet,
        }
    }

    /// Create config for nextnet (experimental)
    pub fn nextnet() -> Self {
        Self {
            base_node_grpc: "http://127.0.0.1:18144".to_string(),
            wallet_grpc: Some("http://127.0.0.1:18144".to_string()),
            network: TariNetwork::Nextnet,
        }
    }

    /// Create custom config
    pub fn custom(base_node_grpc: String, wallet_grpc: Option<String>, network: TariNetwork) -> Self {
        Self {
            base_node_grpc,
            wallet_grpc,
            network,
        }
    }

    /// Get the network name as a string
    pub fn network_name(&self) -> &str {
        match self.network {
            TariNetwork::Mainnet => "mainnet",
            TariNetwork::Esmeralda => "esmeralda",
            TariNetwork::Nextnet => "nextnet",
            TariNetwork::Localnet => "localnet",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = L1Config::default();
        assert_eq!(config.network, TariNetwork::Esmeralda);
        assert_eq!(config.base_node_grpc, "http://127.0.0.1:18143");
    }

    #[test]
    fn test_testnet_config() {
        let config = L1Config::testnet();
        assert_eq!(config.network, TariNetwork::Esmeralda);
        assert_eq!(config.base_node_grpc, "http://127.0.0.1:18143");
    }

    #[test]
    fn test_mainnet_config() {
        let config = L1Config::mainnet();
        assert_eq!(config.network, TariNetwork::Mainnet);
        assert_eq!(config.base_node_grpc, "http://127.0.0.1:18142");
    }

    #[test]
    fn test_custom_config() {
        let config = L1Config::custom(
            "http://custom:1234".to_string(),
            None,
            TariNetwork::Localnet,
        );
        assert_eq!(config.network, TariNetwork::Localnet);
        assert_eq!(config.base_node_grpc, "http://custom:1234");
        assert_eq!(config.wallet_grpc, None);
    }
}
