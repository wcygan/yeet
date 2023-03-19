use crate::args;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, ToServer, UdpConnection};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tub::Pool;

pub struct ServerHandle {
    chan: tokio::sync::mpsc::Sender<(ToServer, SocketAddr)>,
}

struct Server {
    listener: UdpConnection,
    pool: Arc<Pool<UdpConnection>>,
    chan: tokio::sync::mpsc::Receiver<(ToServer, SocketAddr)>,
}

impl ServerHandle {
    pub async fn new() -> Result<Self> {
        let (mut conn, pool) = init().await?;
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let server = Server {
            listener: conn,
            pool,
            chan: rx,
        };

        tokio::spawn(async move { server.run().await });

        Ok(Self { chan: tx })
    }
}

impl Server {
    pub async fn run(mut self) {
        while let Some((message, addr)) = self.chan.recv().await {
            match message {
                ToServer::Join { .. } => {
                    todo!()
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

async fn init() -> Result<(UdpConnection, Arc<Pool<UdpConnection>>)> {
    let args = args::Args::parse();
    let mut writers: Vec<UdpConnection> = vec![];
    for _ in 0..10 {
        writers.push(UdpConnection::new(get_socket(None).await?))
    }

    let listener = get_socket(Some(args.address)).await?;
    let addr = listener.local_addr()?;
    println!("Server started on {}", addr);

    Ok((UdpConnection::new(listener), Arc::new(writers.into())))
}

async fn get_socket(addr: Option<String>) -> Result<UdpSocket> {
    let socket = match addr {
        None => UdpSocket::bind("0.0.0.0:0").await?,
        Some(a) => UdpSocket::bind(a).await?,
    };
    Ok(socket)
}
