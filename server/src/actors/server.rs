use crate::actors::client::ClientHandle;
use crate::args;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, ToServer};
use sockit::UdpSocket;

use crate::time::{next_instant, EXPIRATION_TIME};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::time::Instant;
use tokio_utils::{Pool, ShutdownController, ShutdownMonitor};

static THREE_SECONDS: Duration = Duration::from_secs(3);

pub struct Listener {
    shutdown: ShutdownMonitor,
    socket: UdpSocket,
    chan: tokio::sync::mpsc::Sender<(ToServer, SocketAddr)>,
}

struct Processor {
    server_addr: SocketAddr,
    shutdown: ShutdownMonitor,
    pool: Arc<Pool<UdpSocket>>,
    chan: tokio::sync::mpsc::Receiver<(ToServer, SocketAddr)>,
    clients: HashMap<SocketAddr, ClientHandle>,
}

impl Listener {
    pub async fn new(shutdown: &ShutdownController) -> Result<Self> {
        let (listener, pool) = init().await?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let server_addr = listener.local_addr()?;
        let processor = Processor {
            server_addr,
            shutdown: shutdown.subscribe(),
            pool,
            chan: rx,
            clients: HashMap::new(),
        };

        tokio::spawn(async move { processor.run().await });

        Ok(Self {
            shutdown: shutdown.subscribe(),
            socket: listener,
            chan: tx,
        })
    }

    pub async fn listen(&mut self) {
        while !self.shutdown.is_shutdown() {
            select! {
                _ = self.shutdown.recv() => {
                    println!("server listener shutting down")
                }
                res = self.socket.read::<ToServer>() => {
                    match res {
                        Ok(tup) => {
                            // Pass the message to the processor
                            // So we can continue listening for
                            // incoming messages
                            let _ = self.chan.send(tup).await;
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
}

impl Processor {
    async fn run(mut self) {
        let mut heartbeat_timer =
            tokio::time::interval_at(Instant::now() + THREE_SECONDS, THREE_SECONDS);
        while !self.shutdown.is_shutdown() {
            select! {
                _ = heartbeat_timer.tick() => {
                    let now = Instant::now();
                    self.send_all(self.server_addr, FromServer::Heartbeat).await;
                    let mut peers_to_remove = vec![];

                    for peer in self.clients.values_mut() {
                        if peer.expires < now {
                            println!("Killing {}", peer.name());
                            peers_to_remove.push(peer.addr);
                            kill(self.server_addr, peer).await;
                        }
                    }

                    for peer in peers_to_remove {
                        match self.clients.remove(&peer) {
                            Some(_) => {}
                            None => {println!("Couldn't remove {}", peer)}
                        };
                    }

                }
                _ = self.shutdown.recv() => {
                    println!("server processor shutting down");
                    self.tell_clients_to_shutdown().await;
                },
                option = self.chan.recv() => {
                    if let Some((message, addr)) = option {
                        match message {
                            ToServer::Join { name } => {
                                println!("Sending ACK to {} for joining", name);
                                let pool = self.pool.clone();
                                let client = ClientHandle::new(name.clone(), addr, pool);

                                // Add the client
                                self.clients.insert(addr, client);

                                // Send acknowledgement
                                let _ = self
                                    .pool
                                    .acquire()
                                    .await
                                    .write::<FromServer>(&FromServer::Ack, addr)
                                    .await;

                                let join_msg = format!("{} joined", name);
                                println!("{}", join_msg);
                                let m = FromServer::message(join_msg);
                                self.send_all(addr, m).await;
                            }
                            ToServer::Message { message } => {
                                if let Some(client) = self.clients.get(&addr) {
                                    let m = format!("{}: {}", client.name(), message);
                                    let m = FromServer::message(m);
                                    self.send_all(addr, m).await;
                                }
                            }
                            ToServer::Leave => match self.clients.remove(&addr) {
                                None => {}
                                Some(client) => {
                                    let name = client.name();
                                    let s = format!("{} left", name);
                                    println!("{}", s);

                                    let s = FromServer::message(s);
                                    self.send_all(addr, s).await;
                                }
                            },
                            ToServer::KeepAlive => {
                                if let Some(peer) = self.clients.get_mut(&addr) {
                                    peer.expires = next_instant(peer.expires, EXPIRATION_TIME);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn send_all(&mut self, from: SocketAddr, message: FromServer) {
        for peer in self.clients.values_mut() {
            peer.send(from, message.clone()).await;
        }
    }

    async fn tell_clients_to_shutdown(&mut self) {
        for peer in self.clients.values_mut() {
            kill(self.server_addr, peer).await
        }
    }
}

async fn kill(from: SocketAddr, client: &mut ClientHandle) {
    client.send(from, FromServer::Shutdown).await;
}

async fn init() -> Result<(UdpSocket, Arc<Pool<UdpSocket>>)> {
    let args = args::Args::parse();
    let mut writers: Vec<UdpSocket> = vec![];
    for _ in 0..10 {
        writers.push(get_socket(None).await?)
    }

    let listener = get_socket(Some(args.address)).await?;
    let addr = listener.local_addr()?;
    println!("Server started on {}", addr);

    Ok((listener, Arc::new(writers.into())))
}

async fn get_socket(addr: Option<String>) -> Result<UdpSocket> {
    let socket = match addr {
        None => UdpSocket::bind("0.0.0.0:0").await?,
        Some(a) => UdpSocket::bind(a).await?,
    };
    Ok(socket)
}
