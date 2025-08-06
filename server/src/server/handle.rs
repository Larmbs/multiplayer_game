use std::sync::atomic::{AtomicU64, Ordering};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::cli::ServerConfig;
use super::ServerCommand;
use common::message::{ServerMessage, ClientMessage};
use common::world::{Player, World};

pub struct ClientHandle {
    player_id_counter: Arc<AtomicU64>,
    client_id: Option<u64>,

    // Reference to server config variables
    server_config: Arc<ServerConfig>,

    stream: TcpStream,
    // Sends a ServerCommand to the server to execute it
    tx: UnboundedSender<ServerCommand>,
    // Receives a server message to send to the client
    rx: UnboundedReceiver<ServerMessage>,

    world: Arc<Mutex<World>>,
}

impl ClientHandle {
    pub fn new(
        player_id_counter: Arc<AtomicU64>,
        server_config: Arc<ServerConfig>,
        stream: TcpStream,
        tx: UnboundedSender<ServerCommand>,
        rx: UnboundedReceiver<ServerMessage>,
        world: Arc<Mutex<World>>,
    ) -> Self {
        Self {
            server_config,
            player_id_counter,
            client_id: None,
            stream,
            tx,
            rx,
            world,
        }
    }

    pub async fn handle(&mut self) -> anyhow::Result<()> {
        let mut buf = [0; 1024];

        loop {
            select! {
                n = self.stream.read(&mut buf) => {
                    let n = n?;

                    if n == 0 {
                        println!("Connection closed by peer");
                        break;
                    }

                    let received = &buf[..n];
                    let (client_message, _len) = ClientMessage::decode(received)?;

                    match client_message {
                        ClientMessage::Ping=>{
                            let _ = self.tx.send(ServerCommand::Broadcast(ServerMessage::Ping));
                        },
                        ClientMessage::Connect(username, password) => {
                            if self.server_config.password.is_none() || password == self.server_config.password.clone().unwrap() {
                                let new_id = self.player_id_counter.fetch_add(1, Ordering::Relaxed);
                            let new_player = Player {
                                id: new_id,
                                username,
                                x: 0.0,
                                y: 0.0,
                                vx: 0.0,
                                vy: 0.0,
                            };

                            {
                                let mut world = self.world.lock().await;
                                world.update_player(new_player);
                            }
                            self.client_id = Some(new_id);
                            }
                            // Send ConnectionAccepted with player ID if needed
                    },
                        ClientMessage::NotifyUpdatePlayer(player)=>{
                            // Update the player in the world state
                            let mut world = self.world.lock().await;
                            world.update_player(player);

                            // Broadcast updated players to all clients
                            let players = world.get_all_players().clone();
                            let msg = ServerMessage::UpdatePlayers(players);

                            let _ = self.tx.send(ServerCommand::Broadcast(msg));
                        },
                        ClientMessage::NotifyShot(player)=>{},
                        ClientMessage::Disconnect => {
                            break;
                        },
                    }
                }

                Some(msg) = self.rx.recv() => {
                    let response_bytes = msg.encode()?;
                    self.stream.write_all(&response_bytes).await?;
                }
            }
        }

        Ok(())
    }
}
