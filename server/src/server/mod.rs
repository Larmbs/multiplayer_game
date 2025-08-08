use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time;

use crate::cli::ServerConfig;
use common::message::ServerMessage;
use common::world::World;

mod handle;
use handle::ClientHandle;

enum ServerCommand {
    Broadcast(ServerMessage),
}

pub struct Server {
    listener: TcpListener,

    /* Identification and settings */
    player_id_counter: Arc<AtomicU64>,
    server_config: Arc<ServerConfig>,

    /* Communication between server and client handles */
    client_txs: Arc<Mutex<Vec<UnboundedSender<ServerMessage>>>>,
    command_rx: UnboundedReceiver<ServerCommand>,
    command_tx: UnboundedSender<ServerCommand>,

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
        let world = self.world.clone();
        let command_tx = self.command_tx.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(50)); // update every 50 ms
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
                    }
                }
            }
        }
    }
}
