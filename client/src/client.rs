use anyhow::Result;
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
        username: String,
        password: String,
        runtime_tx: UnboundedSender<ServerMessage>,
        runtime_rx: UnboundedReceiver<ClientMessage>,
    ) -> anyhow::Result<(u64, Self)> {
        let mut stream = TcpStream::connect(addr).await?;
        println!("Connected to {}", stream.peer_addr()?);

        let response_bytes = ClientMessage::Connect(username, password).encode()?;
        stream.write_all(&response_bytes).await?;
        // Read initial response (waiting for ConnectionAccepted)
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).await?;

        if n == 0 {
            anyhow::bail!("Server closed the connection");
        }

        let received = &buf[..n];
        let (msg, _) = ServerMessage::decode(received)?;
        let player_id = match msg {
            ServerMessage::ConnectionAccepted(id) => {
                println!("Connection accepted. Player ID: {}", id);
                id
            }
            other => anyhow::bail!("Expected ConnectionAccepted, got: {:?}", other),
        };

        Ok((
            player_id,
            Self {
                stream,
                runtime_tx,
                runtime_rx,
            },
        ))
    }

    // Send a client message to the server
    pub async fn send_message(&mut self, msg: ClientMessage) -> anyhow::Result<()> {
        let encoded = msg.encode()?;
        self.stream.write_all(&encoded).await?;
        Ok(())
    }

    pub async fn listen(&mut self) -> Result<()> {
        let mut read_buf = [0u8; 4096];
        let mut read_pos = 0;

        loop {
            tokio::select! {
                // 1) Read from the server
                read_res = self.stream.read(&mut read_buf[read_pos..]) => {
                    let n = read_res?;
                    if n == 0 {
                        println!("Server closed connection");
                        break;
                    }
                    read_pos += n;

                    // Try to decode as many ServerMessages as possible from buffer
                    let mut offset = 0;
                    while offset < read_pos {
                        match ServerMessage::decode(&read_buf[offset..read_pos]) {
                            Ok((msg, len)) => {
                                self.runtime_tx.send(msg).ok(); // Ignore send errors (runtime dropped)
                                offset += len;
                            }
                            Err(_) => {
                                // Incomplete message? Wait for more bytes
                                break;
                            }
                        }
                    }

                    // Remove consumed bytes from buffer by shifting remaining to start
                    if offset > 0 {
                        read_buf.copy_within(offset..read_pos, 0);
                        read_pos -= offset;
                    }
                }

                // 2) Receive outgoing messages from runtime and send to server
                Some(msg) = self.runtime_rx.recv() => {
                    self.send_message(msg).await?;
                }
            }
        }

        Ok(())
    }
}
