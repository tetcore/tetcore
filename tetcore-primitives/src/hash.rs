// File: hash.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Hash primitives for Tetcore providing Hash32, a 256-bit (32-byte)
// hash type used throughout the runtime for block hashing, state roots,
// transaction IDs, and cryptographic commitments. Implements canonical
// encoding for deterministic serialization.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Default, PartialEq, Eq, Hash, Copy)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub const SIZE: usize = 32;

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        let len = slice.len().min(32);
        hash[..len].copy_from_slice(&slice[..len]);
        Self(hash)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let bytes = hex::decode(hex_str).map_err(|e| e.to_string())?;
        if bytes.len() != 32 {
            return Err("Invalid hash length".to_string());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash32(arr))
    }

    pub fn empty() -> Self {
        Self([0u8; 32])
    }

    pub fn reverse(self) -> Self {
        let mut hash = self.0;
        hash.reverse();
        Self(hash)
    }
}

impl AsRef<[u8]> for Hash32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; 32]> for Hash32 {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for Hash32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl std::fmt::Debug for Hash32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hash32(0x{})", self.to_hex())
    }
}

impl Serialize for Hash32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Hash32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid hash length"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash32(arr))
    }
}

impl From<[u8; 32]> for Hash32 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for Hash32 {
    type Error = &'static str;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() < 32 {
            return Err("Slice too short");
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&slice[..32]);
        Ok(Hash32(arr))
    }
}

impl TryFrom<Vec<u8>> for Hash32 {
    type Error = &'static str;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(vec.as_slice())
    }
}
