//! Handles the client connections and communication with the server.
use anyhow::Result;
use std::sync::Arc;

use tokio::{
    net::TcpStream,
    select,
    sync::{
        Mutex,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
};

use super::ServerCommand;
use crate::cli::ServerConfig;
use common::world::{World, entities::Player};
use common::{
    color::Color,
    message::{ClientMessage, ServerMessage},
    vec::Vec2,
};

/// ClientHandle manages a single client connection, processing messages and updating the game state.
/// It handles incoming messages from the client, updates the world state, and sends responses back to
pub struct ClientHandle {
    /// TCP stream for communication with the client
    stream: TcpStream,

    /// Unique identifier for the client
    client_id: u64,
    /// Whether the client has been accepted and is allowed to interact with the server
    accepted: bool,

    // Reference to server config variables
    server_config: Arc<ServerConfig>,

    /* Server Handle communication */
    /// Sends a ServerCommand to the server to execute it
    tx: UnboundedSender<ServerCommand>,
    /// Receives a server message to send to the client
    rx: UnboundedReceiver<ServerMessage>,

    world: Arc<Mutex<World>>,
}

impl ClientHandle {
    pub fn new(
        client_id: u64,
        server_config: Arc<ServerConfig>,
        stream: TcpStream,
        tx: UnboundedSender<ServerCommand>,
        rx: UnboundedReceiver<ServerMessage>,
        world: Arc<Mutex<World>>,
    ) -> Self {
        Self {
            server_config,
            client_id,
            stream,
            tx,
            rx,
            world,
            accepted: false,
        }
    }

    /// Handles the client connection, processing messages and updating the world state.
    pub async fn handle(&mut self) -> Result<()> {
        let mut buffer = [0; 1024];

        loop {
            select! {
                client_message = ClientMessage::read_from_tcp_stream(&mut self.stream, &mut buffer) => {
                    match client_message? {
                        ClientMessage::Ping =>{
                            let _ = self.tx.send(ServerCommand::Broadcast(ServerMessage::Ping));
                        },
                        ClientMessage::Connect(username, password) => {
                            // Check if the password is correct
                            if self.server_config.password.is_none() || password == self.server_config.password.clone().unwrap() {
                                // Create a new player and add it to the world
                                let new_player = Player {
                                    username,
                                    color: Color::random(), // Default color
                                    pos: Vec2::ZERO,
                                    vel: Vec2::ZERO,
                                };
                                let mut world = self.world.lock().await;
                                world.entities.players.insert(self.client_id, new_player);

                                let _ = ServerMessage::ConnectionAccepted(self.client_id).write_to_tcp_stream(&mut self.stream).await;
                                let _ = self.tx.send(ServerCommand::UpdateEntities);

                                self.accepted = true;
                            }
                        },
                        ClientMessage::NotifyUpdatePlayer(player) =>{
                            // Update the player in the world state
                            let mut world = self.world.lock().await;
                            world.entities.players.insert(self.client_id, player);

                            // Broadcast updated players to all clients
                            let _ = self.tx.send(ServerCommand::UpdateEntities);
                        },
                        ClientMessage::Disconnect => {
                            let mut world = self.world.lock().await;
                            world.entities.players.remove(&self.client_id);

                            let _ = self.tx.send(ServerCommand::UpdateEntities);
                            break;
                        },
                    }
                }
                Some(msg) = self.rx.recv() => {
                    if self.accepted {
                        let _ = msg.write_to_tcp_stream(&mut self.stream).await;
                    }
                }
            }
        }

        Ok(())
    }
}
