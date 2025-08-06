//! Defines how the server and client communicate
//!
//! This module provides `ServerMessage` and `ClientMessage` enums which define
//! the types of messages that can be exchanged between a server and a client.
//! These messages can be serialized and deserialized using `bincode` for efficient
//! binary communication. Each message type implements `encode` and `decode` methods
//! to handle this serialization logic.

use anyhow::Result;
use bincode::{Decode, Encode, config};
use serde::{Deserialize, Serialize};

/// Messages that are sent from the Client to the Server 
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub enum ServerMessage {
    Ping,
}
impl ServerMessage {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let config = config::standard();
        Ok(bincode::encode_to_vec(self, config)?)
    }
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize)> {
        let config = config::standard();
        Ok(bincode::decode_from_slice(bytes, config)?)
    }
}

/// Messages that are sent from the Client to the Server 
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub enum ClientMessage {
    Ping,
}
impl ClientMessage {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let config = config::standard();
        Ok(bincode::encode_to_vec(self, config)?)
    }
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize)> {
        let config = config::standard();
        Ok(bincode::decode_from_slice(bytes, config)?)
    }
}
