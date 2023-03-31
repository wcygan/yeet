use crate::time::{next_instant, EXPIRATION_TIME};
use common::FromServer;
use sockit::UdpSocket;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Instant;
use tokio_utils::Pool;

pub struct ClientHandle {
    pub expires: Instant,
    name: String,
    pub addr: SocketAddr,
    chan: tokio::sync::mpsc::Sender<FromServer>,
}

struct Client {
    addr: SocketAddr,
    pool: Arc<Pool<UdpSocket>>,
    chan: tokio::sync::mpsc::Receiver<FromServer>,
}

impl ClientHandle {
    pub fn new(name: String, addr: SocketAddr, pool: Arc<Pool<UdpSocket>>) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let client = Client::new(addr, pool, rx);
        tokio::spawn(async move {
            client.run().await;
        });

        Self {
            expires: next_instant(Instant::now(), EXPIRATION_TIME),
            name,
            addr,
            chan: tx,
        }
    }

    pub async fn send(&mut self, from: SocketAddr, msg: FromServer) {
        if from != self.addr {
            let _ = self.chan.send(msg).await;
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl Client {
    fn new(
        addr: SocketAddr,
        pool: Arc<Pool<UdpSocket>>,
        chan: tokio::sync::mpsc::Receiver<FromServer>,
    ) -> Self {
        Self { addr, pool, chan }
    }

    async fn run(mut self) {
        while let Some(msg) = self.chan.recv().await {
            let _ = self
                .pool
                .acquire()
                .await
                .write::<FromServer>(&msg, self.addr)
                .await;
        }
    }
}
