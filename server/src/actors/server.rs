use crate::actors::client::ClientHandle;
use crate::args;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use dashmap::DashMap;
use lib_wc::sync::{ShutdownController, ShutdownListener};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::select;
use tub::Pool;

pub struct Listener {
    shutdown: ShutdownListener,
    socket: Socket,
    chan: tokio::sync::mpsc::Sender<(ToServer, SocketAddr)>,
}

struct Processor {
    shutdown: ShutdownListener,
    pool: Arc<Pool<Socket>>,
    chan: tokio::sync::mpsc::Receiver<(ToServer, SocketAddr)>,
    clients: HashMap<SocketAddr, ClientHandle>,
}

impl Listener {
    pub async fn new(shutdown: &ShutdownController) -> Result<Self> {
        let (mut listener, pool) = init().await?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let processor = Processor {
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
                    println!("x");
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
        // TODO: Add a shutdown handler
        // TODO: select loop:
        //       1. heartbeat clients every 5 seconds
        //         a. Set a TTL on clients and remove them if they don't respond quickly enough
        //       2. wait for incoming messages

        while !self.shutdown.is_shutdown() {
            select! {
                _ = self.shutdown.recv() => {
                    println!("server processor shutting down")
                },
                option = self.chan.recv() => {
                    println!("d");
                    if let Some((message, addr)) = option {
                        match message {
                            ToServer::Join { name } => {
                                let join_msg = format!("{} joined", name);
                                self.send_all(addr, join_msg).await;

                                let pool = self.pool.clone();
                                let client = ClientHandle::new(name, addr, pool);

                                // Add the client
                                self.clients.insert(addr, client);

                                // Send acknowledgement
                                let _ = self
                                    .pool
                                    .acquire()
                                    .await
                                    .write::<FromServer>(&FromServer::Ack, addr)
                                    .await;
                            }
                            ToServer::Message { message } => {
                                if let Some(client) = self.clients.get(&addr) {
                                    let m = format!("{}: {}", client.name(), message);
                                    self.send_all(addr, m).await;
                                }
                            }
                            ToServer::Leave => match self.clients.remove(&addr) {
                                None => {}
                                Some(client) => {
                                    let name = client.name();
                                    let s = format!("{} left", name);
                                    println!("{}", s);

                                    self.send_all(addr, s).await;
                                }
                            },
                            ToServer::Pong => {
                                todo!()
                            }
                        }
                    }
                }
            }
        }
    }

    async fn send_all(&mut self, from: SocketAddr, m: String) {
        let e = FromServer::Message { message: m };

        for (_peer_addr, mut peer) in &mut self.clients {
            println!("Sending {:?} to {}", e, peer.name());
            peer.send(from, e.clone()).await;
        }
    }
}

async fn init() -> Result<(Socket, Arc<Pool<Socket>>)> {
    let args = args::Args::parse();
    let mut writers: Vec<Socket> = vec![];
    for _ in 0..10 {
        writers.push(Socket::new(get_socket(None).await?))
    }

    let listener = get_socket(Some(args.address)).await?;
    let addr = listener.local_addr()?;
    println!("Server started on {}", addr);

    Ok((Socket::new(listener), Arc::new(writers.into())))
}

async fn get_socket(addr: Option<String>) -> Result<UdpSocket> {
    let socket = match addr {
        None => UdpSocket::bind("0.0.0.0:0").await?,
        Some(a) => UdpSocket::bind(a).await?,
    };
    Ok(socket)
}
