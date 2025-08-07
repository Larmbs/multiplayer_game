//! Defines how the server and client communicate
//!
//! This module provides `ServerMessage` and `ClientMessage` enums which define
//! the types of messages that can be exchanged between a server and a client.
//! These messages can be serialized and deserialized using `bincode` for efficient
//! binary communication. Each message type implements `encode` and `decode` methods
//! to handle this serialization logic.

use std::collections::HashMap;

use anyhow::Result;
use bincode::{Decode, Encode, config};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::world::{Player, Projectile};

/// Messages that are sent from the Server to the Client
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub enum ServerMessage {
    /* Connection handling */
    Ping,
    Disconnect,
    ConnectionAccepted(u64),
    PasswordFailed,

    /* Notifies players of world updates */
    UpdatePlayers(HashMap<u64, Player>),
    UpdateProjectiles(Vec<Projectile>),
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
impl ServerMessage {
    pub async fn write_to_tcp_stream(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let encoded = self.encode()?;
        let len = encoded.len() as u32;
        stream.write_u32(len).await?;
        stream.write_all(&encoded).await?;
        Ok(())
    }
    pub async fn read_from_tcp_stream(
        stream: &mut TcpStream,
        buffer: &mut [u8; 1024],
    ) -> anyhow::Result<Self> {
        let size = stream.read(buffer).await?;
        if size == 0 {
            Ok(ServerMessage::Disconnect)
        } else {
            let (msg, _) = ServerMessage::decode(buffer)?;
            Ok(msg)
        }
    }
}

/// Messages that are sent from the Client to the Server
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub enum ClientMessage {
    /* Connection handling */
    /// Username, Password (If user name is duplicate it will assign you an new one)
    Connect(String, String),
    Disconnect,
    Ping,

    /* Notifies server of client updates */
    NotifyUpdatePlayer(Player),
    NotifyShot(Projectile),
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
impl ClientMessage {
    pub async fn write_to_tcp_stream(&self, stream: &mut TcpStream) -> anyhow::Result<()> {
        let encoded = self.encode()?;
        let len = encoded.len() as u32;
        stream.write_u32(len).await?;
        stream.write_all(&encoded).await?;
        Ok(())
    }

    pub async fn read_from_tcp_stream(
        stream: &mut TcpStream,
        buffer: &mut [u8; 1024],
    ) -> anyhow::Result<Self> {
        let size = stream.read(buffer).await?;
        if size == 0 {
            Ok(ClientMessage::Disconnect)
        } else {
            let (msg, _) = ClientMessage::decode(buffer)?;
            Ok(msg)
        }
    }
}
