//! A TCP-based asynchronous server that accepts client connections,
//! handles message exchange using a custom protocol, and supports
//! commands like broadcasting messages and querying world state.
//!
//! This module defines the main `Server` structure, a per-client
//! `ClientHandle`, and a `ServerCommand` enum for internal coordination.
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::cli::ServerConfig;
use common::message::{ClientMessage, ServerMessage};
use common::world::World;

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
    server_config: ServerConfig,
    listener: TcpListener,
    clients: Arc<Mutex<Vec<UnboundedSender<ServerMessage>>>>,
    command_receiver: UnboundedReceiver<ServerCommand>,
    command_sender: UnboundedSender<ServerCommand>,

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
            server_config,
            listener,
            clients: Arc::new(Mutex::new(vec![])),
            command_receiver: rx,
            command_sender: tx,

            world: Arc::new(Mutex::new(World::new())),
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

                    if self.clients.lock().await.len() < self.server_config.max_clients {
                        let (tx_to_client, rx_for_client) = unbounded_channel::<ServerMessage>();
                        let client_command_sender = self.command_sender.clone();
                        self.clients.lock().await.push(tx_to_client.clone());

                        let mut client = ClientHandle::new(stream, client_command_sender, rx_for_client);

                        tokio::spawn(async move {
                            if let Err(e) = client.handle().await {
                                eprintln!("Client error: {:?}", e);
                            }
                        });
                    }
                }

                Some(cmd) = self.command_receiver.recv() => {
                    match cmd {
                        ServerCommand::Broadcast(msg) => {
                            let clients = self.clients.lock().await;
                            for tx in clients.iter() {
                                let _ = tx.send(msg.clone());
                            }
                        }
                        ServerCommand::PingAll => {
                            let clients = self.clients.lock().await;
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
    stream: TcpStream,
    // Sends a ServerCommand to the server to execute it
    tx: UnboundedSender<ServerCommand>,
    // Receives a server message to send to the client
    rx: UnboundedReceiver<ServerMessage>,
}

impl ClientHandle {
    fn new(
        stream: TcpStream,
        tx: UnboundedSender<ServerCommand>,
        rx: UnboundedReceiver<ServerMessage>,
    ) -> Self {
        Self { stream, tx, rx }
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
                        ClientMessage::Ping => {
                            let _ = self.tx.send(ServerCommand::PingAll); // Or custom logic
                        },
                        // Do nothing client has already been accepted
                        ClientMessage::Connect(_, _) => (),
                        ClientMessage::NotifyUpdatePlayer(player) => {

                        },
                        ClientMessage::NotifyShot(player) => {

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
