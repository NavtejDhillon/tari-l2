use serde::{Deserialize, Serialize};
use std::fmt;

/// Hash type for all hashing operations
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 32 {
            return Err("Invalid hash length");
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

/// Public key type
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 32 {
            return Err("Invalid public key length");
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PublicKey({})", hex::encode(&self.0[..8]))
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

/// Signature type
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "serde_arrays")] [u8; 64]);

impl Signature {
    pub fn new(data: [u8; 64]) -> Self {
        Self(data)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 64 {
            return Err("Invalid signature length");
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(slice);
        Ok(Self(arr))
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Signature({})", hex::encode(&self.0[..8]))
    }
}

/// Amount type for representing token amounts
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub struct Amount(u64);

impl Amount {
    pub const ZERO: Amount = Amount(0);

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn checked_add(&self, other: Amount) -> Option<Amount> {
        self.0.checked_add(other.0).map(Amount)
    }

    pub fn checked_sub(&self, other: Amount) -> Option<Amount> {
        self.0.checked_sub(other.0).map(Amount)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Timestamp type
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp() as u64)
    }

    pub fn from_secs(secs: u64) -> Self {
        Self(secs)
    }

    pub fn as_secs(&self) -> u64 {
        self.0
    }
}
