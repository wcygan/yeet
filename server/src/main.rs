use anyhow::Result;
use common::{FromServer, ToServer, UdpConnection};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tub::Pool;

#[tokio::main]
async fn main() -> Result<()> {
    let (listener, pool) = get_sockets().await?;
    let mut conn = UdpConnection::new(listener);

    loop {
        let (msg, src) = conn.read::<ToServer>().await?;
        let pool = pool.clone();

        tokio::spawn(async move {
            match msg {
                ToServer::Message { message } => {
                    let t = FromServer::Message { message };
                    let mut conn = pool.acquire().await;
                    conn.write::<FromServer>(&t, src).await?;
                }
                _ => {}
            }

            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn get_sockets() -> Result<(UdpSocket, Arc<Pool<UdpConnection>>)> {
    let mut writers: Vec<UdpConnection> = vec![];
    for _ in 0..10 {
        writers.push(UdpConnection::new(bind_any().await?))
    }

    let listener = bind_any().await?;
    let addr = listener.local_addr()?;
    println!("Server started on {}", addr);

    Ok((listener, Arc::new(writers.into())))
}

async fn bind_any() -> Result<UdpSocket> {
    Ok(UdpSocket::bind("0.0.0.0:0").await?)
}
