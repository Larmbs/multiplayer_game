//! A TCP-based asynchronous server that accepts client connections,
//! handles message exchange using a custom protocol, and supports
//! commands like broadcasting messages and querying world state.
//!
//! This module defines the main `Server` structure, a per-client
//! `ClientHandle`, and a `ServerCommand` enum for internal coordination.
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::cli::ServerConfig;
use common::message::{ClientMessage, ServerMessage};
use common::world::{Player, World};

/// Commands sent from client handlers to the central server.
///
/// These commands allow client handlers to request shared operations like
/// broadcasting a message, updating shared state, or triggering world info responses.#[derive(Debug, Clone)]
enum ServerCommand {
    PingAll,
    Broadcast(ServerMessage),
}

/// An asynchronous TCP server that manages client connections and
/// processes commands using Tokio's event-driven runtime.
///
/// The server listens for incoming TCP connections, spawns a handler
/// for each client, and coordinates internal commands via an unbounded channel.
pub struct Server {
    player_id_counter: Arc<AtomicU64>,
    server_config: Arc<ServerConfig>,
    listener: TcpListener,
    client_txs: Arc<Mutex<Vec<UnboundedSender<ServerMessage>>>>,
    command_rx: UnboundedReceiver<ServerCommand>,
    command_tx: UnboundedSender<ServerCommand>,

    // Shared data about game world
    world: Arc<Mutex<World>>,
}

impl Server {
    pub async fn init<T: ToSocketAddrs>(
        addr: T,
        server_config: ServerConfig,
    ) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let (tx, rx) = unbounded_channel();

        Ok(Self {
            server_config: Arc::new(server_config),
            listener,
            client_txs: Arc::new(Mutex::new(vec![])),
            command_rx: rx,
            command_tx: tx,

            world: Arc::new(Mutex::new(World::new())),
            player_id_counter: Arc::new(AtomicU64::new(1)),
        })
    }

    pub fn get_address(&self) -> String {
        self.listener
            .local_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| String::from("None"))
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        loop {
            select! {
                Ok((stream, addr)) = self.listener.accept() => {
                    println!("New client: {}", addr);

                    if self.client_txs.lock().await.len() < self.server_config.max_clients {
                        let (tx_to_client, rx_for_client) = unbounded_channel::<ServerMessage>();
                        let client_command_sender = self.command_tx.clone();
                        self.client_txs.lock().await.push(tx_to_client.clone());

                        let mut client = ClientHandle::new(self.player_id_counter.clone(),
                        self.server_config.clone(), stream, client_command_sender, rx_for_client, self.world.clone());

                        tokio::spawn(async move {
                            if let Err(e) = client.handle().await {
                                eprintln!("Client error: {:?}", e);
                            }
                        });
                    }
                }

                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        ServerCommand::Broadcast(msg) => {
                            let clients = self.client_txs.lock().await;
                            for tx in clients.iter() {
                                let _ = tx.send(msg.clone());
                            }
                        }
                        ServerCommand::PingAll => {
                            let clients = self.client_txs.lock().await;
                            for tx in clients.iter() {
                                let _ = tx.send(ServerMessage::Ping);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A per-client handler that manages communication between a single client and the server.
///
/// Each connected client has a dedicated `ClientHandle` which reads messages
/// from the TCP stream, sends commands to the server, and listens for server messages.
struct ClientHandle {
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
    fn new(
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

    async fn handle(&mut self) -> anyhow::Result<()> {
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
                            let _ = self.tx.send(ServerCommand::PingAll);
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
