// File: crypto.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Cryptographic primitives for Tetcore including Address, PublicKey,
// PrivateKey, and Signature types. Provides Ed25519-based key derivation,
// address encoding, and signature verification for transaction authentication.

use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const ADDRESS_PREFIX: &str = "TETCORE:ADDR:v1";
pub const PUBLIC_KEY_SIZE: usize = 32;
pub const PRIVATE_KEY_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 64;
pub const ADDRESS_SIZE: usize = 32;

#[derive(Error, Debug, Clone)]
pub enum CryptoError {
    #[error("Invalid public key length")]
    InvalidPublicKeyLength,
    #[error("Invalid private key length")]
    InvalidPrivateKeyLength,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid address format")]
    InvalidAddressFormat,
    #[error("Signing error")]
    SigningError,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PublicKey([u8; PUBLIC_KEY_SIZE]);

impl PublicKey {
    pub fn from_bytes(bytes: [u8; PUBLIC_KEY_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn from_verifying_key(key: VerifyingKey) -> Self {
        Self(key.to_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_SIZE] {
        &self.0
    }

    pub fn to_verifying_key(&self) -> Result<VerifyingKey, CryptoError> {
        VerifyingKey::from_bytes(&self.0).map_err(|_| CryptoError::InvalidPublicKeyLength)
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != PUBLIC_KEY_SIZE {
            return Err(serde::de::Error::custom("Invalid public key length"));
        }
        let mut arr = [0u8; PUBLIC_KEY_SIZE];
        arr.copy_from_slice(&bytes);
        Ok(PublicKey(arr))
    }
}

#[derive(Clone, Debug)]
pub struct PrivateKey(pub [u8; PRIVATE_KEY_SIZE]);

impl PrivateKey {
    pub fn from_bytes(bytes: [u8; PRIVATE_KEY_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn from_signing_key(key: SigningKey) -> Self {
        Self(key.to_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; PRIVATE_KEY_SIZE] {
        &self.0
    }

    pub fn to_signing_key(&self) -> Result<SigningKey, CryptoError> {
        Ok(SigningKey::from_bytes(&self.0))
    }

    pub fn public_key(&self) -> PublicKey {
        let signing_key = self.to_signing_key().unwrap();
        PublicKey::from_verifying_key(signing_key.verifying_key())
    }
}

impl serde::Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> serde::Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != PRIVATE_KEY_SIZE {
            return Err(serde::de::Error::custom("Invalid private key length"));
        }
        let mut arr = [0u8; PRIVATE_KEY_SIZE];
        arr.copy_from_slice(&bytes);
        Ok(PrivateKey(arr))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Copy)]
pub struct Address([u8; ADDRESS_SIZE]);

impl Address {
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(ADDRESS_PREFIX.as_bytes());
        hasher.update(public_key.as_bytes());
        let result = hasher.finalize();
        let mut addr = [0u8; ADDRESS_SIZE];
        addr.copy_from_slice(&result[..ADDRESS_SIZE]);
        Address(addr)
    }

    pub fn from_bytes(bytes: [u8; ADDRESS_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; ADDRESS_SIZE] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(hex_str).map_err(|_| CryptoError::InvalidAddressFormat)?;
        if bytes.len() != ADDRESS_SIZE {
            return Err(CryptoError::InvalidAddressFormat);
        }
        let mut arr = [0u8; ADDRESS_SIZE];
        arr.copy_from_slice(&bytes);
        Ok(Address(arr))
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl serde::Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != ADDRESS_SIZE {
            return Err(serde::de::Error::custom("Invalid address length"));
        }
        let mut arr = [0u8; ADDRESS_SIZE];
        arr.copy_from_slice(&bytes);
        Ok(Address(arr))
    }
}

#[derive(Clone, Debug)]
pub struct Signature(pub [u8; SIGNATURE_SIZE]);

impl Signature {
    pub fn from_bytes(bytes: [u8; SIGNATURE_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; SIGNATURE_SIZE] {
        &self.0
    }

    pub fn to_ed25519_signature(&self) -> Result<Ed25519Signature, CryptoError> {
        Ok(Ed25519Signature::from_bytes(&self.0))
    }

    pub fn from_ed25519_signature(sig: Ed25519Signature) -> Self {
        let mut bytes = [0u8; SIGNATURE_SIZE];
        bytes.copy_from_slice(&sig.to_bytes());
        Self(bytes)
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != SIGNATURE_SIZE {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }
        let mut arr = [0u8; SIGNATURE_SIZE];
        arr.copy_from_slice(&bytes);
        Ok(Signature(arr))
    }
}

pub fn sign(private_key: &PrivateKey, message: &[u8]) -> Result<Signature, CryptoError> {
    let signing_key = private_key
        .to_signing_key()
        .map_err(|_| CryptoError::SigningError)?;
    let sig = signing_key.sign(message);
    Ok(Signature::from_ed25519_signature(sig))
}

pub fn verify(
    public_key: &PublicKey,
    message: &[u8],
    signature: &Signature,
) -> Result<bool, CryptoError> {
    let verifying_key = public_key.to_verifying_key()?;
    let ed25519_sig = signature.to_ed25519_signature()?;
    Ok(verifying_key.verify(message, &ed25519_sig).is_ok())
}

#[cfg(feature = "std")]
impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}
