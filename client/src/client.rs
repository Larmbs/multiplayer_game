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

        ClientMessage::Connect(username, password)
            .write_to_tcp_stream(&mut stream)
            .await?;

        let mut buffer = [0; 1024];
        match ServerMessage::read_from_tcp_stream(&mut stream, &mut buffer).await? {
            ServerMessage::ConnectionAccepted(id) => Ok((
                id,
                Self {
                    stream,
                    runtime_tx,
                    runtime_rx,
                },
            )),
            _ => {
                println!("Error");
                Err(anyhow::anyhow!("Error"))
            }
        }
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
