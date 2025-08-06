use common::message::{ClientMessage, ServerMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Client {
    stream: TcpStream,
    runtime_tx: UnboundedSender<ServerMessage>,
    runtime_rx: UnboundedReceiver<ClientMessage>,
}

impl Client {
    // Connect to the server at the given address
    pub async fn connect<T: ToSocketAddrs>(
        addr: T,
        runtime_tx: UnboundedSender<ServerMessage>,
        runtime_rx: UnboundedReceiver<ClientMessage>,
    ) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        println!("Connected to {}", stream.peer_addr()?);
        Ok(Self { stream, runtime_tx, runtime_rx })
    }

    // Send a client message to the server
    pub async fn send_message(&mut self, msg: ClientMessage) -> anyhow::Result<()> {
        let encoded = msg.encode()?;
        self.stream.write_all(&encoded).await?;
        Ok(())
    }

    // Listen for incoming messages from the server indefinitely
    pub async fn listen(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];
        loop {
            let n = self.stream.read(&mut buf).await?;
            if n == 0 {
                println!("Server closed connection");
                break;
            }
            let received = &buf[..n];
            match ServerMessage::decode(received) {
                Ok((msg, _len)) => {
                    println!("Received from server: {:?}", msg);
                }
                Err(e) => {
                    eprintln!("Failed to decode server message: {:?}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}
