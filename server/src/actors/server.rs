use crate::args;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tub::Pool;

pub struct Listener {
    socket: Socket,
    chan: tokio::sync::mpsc::Sender<(ToServer, SocketAddr)>,
}

struct Processor {
    pool: Arc<Pool<Socket>>,
    chan: tokio::sync::mpsc::Receiver<(ToServer, SocketAddr)>,
    clients: DashMap<SocketAddr, String>,
}

impl Listener {
    pub async fn new() -> Result<Self> {
        let (mut listener, pool) = init().await?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let processor = Processor {
            pool,
            chan: rx,
            clients: DashMap::new(),
        };

        tokio::spawn(async move { processor.run().await });

        Ok(Self {
            socket: listener,
            chan: tx,
        })
    }

    pub async fn listen(&mut self) {
        loop {
            match self.socket.read::<ToServer>().await {
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

impl Processor {
    pub async fn run(mut self) {
        while let Some((message, addr)) = self.chan.recv().await {
            match message {
                ToServer::Join { name } => {
                    // TODO: Add a new ClientHandle

                    let _ = self
                        .pool
                        .acquire()
                        .await
                        .write::<FromServer>(&FromServer::Ack, addr)
                        .await;
                }
                ToServer::Message { message } => {
                    let e = FromServer::Message { message };

                    let _ = self
                        .pool
                        .acquire()
                        .await
                        .write::<FromServer>(&e, addr)
                        .await;
                }
                ToServer::Leave => {
                    todo!()
                }
                ToServer::Pong => {
                    todo!()
                }
            }
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
