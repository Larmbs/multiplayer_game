//! Server module for handling multiplayer game connections, world state, and client communication.
//!
//! This module provides the [`Server`] struct, which manages TCP connections from clients,
//! maintains the shared game world state, and broadcasts updates to all connected clients.
//! It uses asynchronous Tokio primitives for concurrency and message passing between the server
//! and client handlers. The server supports a configurable maximum number of clients and
//! periodically updates and synchronizes the world state.

use anyhow::Result;
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    select,
    sync::{
        Mutex,
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    },
    time,
};

mod handle;

use crate::cli::ServerConfig;
use common::{message::ServerMessage, world::World};
use handle::ClientHandle;

/// Commands that the server can execute that a handle would otherwise not.
enum ServerCommand {
    Broadcast(ServerMessage),
    UpdateEntities,
}

/// Server struct that deploys handles for each client connection and manages the game world.
pub struct Server {
    listener: TcpListener,

    /* Identification and settings */
    player_id_counter: Arc<AtomicU64>,
    server_config: Arc<ServerConfig>,

    /* Communication between server and client handles */
    client_txs: Arc<Mutex<Vec<UnboundedSender<ServerMessage>>>>,
    command_rx: UnboundedReceiver<ServerCommand>,
    command_tx: UnboundedSender<ServerCommand>, // Used for copying to handles

    world: Arc<Mutex<World>>,
}

impl Server {
    pub async fn init<T: ToSocketAddrs>(addr: T, server_config: ServerConfig) -> Result<Self> {
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

    pub async fn run(&mut self) -> Result<()> {
        let world = self.world.clone();
        let command_tx = self.command_tx.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs_f64(1.0 / 60.0));
            loop {
                interval.tick().await;

                {
                    let mut w = world.lock().await;
                    w.entities.update(0.05); // advance the world state by 50 ms (or whatever dt)
                }
                // Broadcast updated world to clients
                // (Here you can customize message type accordingly)
                if let Err(e) = command_tx.send(ServerCommand::Broadcast(
                    ServerMessage::UpdateEntities(world.lock().await.entities.clone()),
                )) {
                    eprintln!("Failed to broadcast world update: {:?}", e);
                }
            }
        });

        loop {
            select! {
                // Accepts connections and creates new client handles
                Ok((stream, addr)) = self.listener.accept() => {
                    println!("New client: {}", addr);

                    if self.client_txs.lock().await.len() < self.server_config.max_clients {
                        let (tx_to_client, rx_for_client) = unbounded_channel();
                        let client_command_sender = self.command_tx.clone();
                        self.client_txs.lock().await.push(tx_to_client.clone());

                        let mut client = ClientHandle::new(
                            self.player_id_counter.fetch_add(1, Ordering::Relaxed),
                            self.server_config.clone(),
                            stream,
                            client_command_sender,
                            rx_for_client,
                            self.world.clone()
                        );

                        tokio::spawn(async move {
                            let _ = client.handle().await;
                        });
                    }
                }
                // Handles commands from server handles
                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        ServerCommand::Broadcast(msg)=>{
                            let clients = self.client_txs.lock().await;
                            for tx in clients.iter() {
                                let _ = tx.send(msg.clone());
                            }
                        }
                        ServerCommand::UpdateEntities => {
                            let clients = self.client_txs.lock().await;
                            let msg = ServerMessage::UpdateEntities(self.world.lock().await.entities.clone());
                            for tx in clients.iter() {
                                let _ = tx.send(msg.clone());
                            }
                        },
                    }
                }
            }
        }
    }
}
impl Server {
    pub fn get_address(&self) -> Option<SocketAddr> {
        self.listener.local_addr().ok()
    }
}
