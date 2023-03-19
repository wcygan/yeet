use anyhow::Result;
use common::{FromServer, ToServer, UdpConnection, DEFAULT_ADDRESS};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tub::Pool;

#[tokio::main]
async fn main() -> Result<()> {
    let (mut conn, pool) = get_sockets(DEFAULT_ADDRESS.into()).await?;

    loop {
        let (msg, src) = conn.read::<ToServer>().await?;
        let pool = pool.clone();

        tokio::spawn(async move {
            match msg {
                ToServer::Message { message } => {
                    println!("{}", message);
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

async fn get_sockets(addr: Option<&str>) -> Result<(UdpConnection, Arc<Pool<UdpConnection>>)> {
    let mut writers: Vec<UdpConnection> = vec![];
    for _ in 0..10 {
        writers.push(UdpConnection::new(get_socket(None).await?))
    }

    let listener = get_socket(addr).await?;
    let addr = listener.local_addr()?;
    println!("Server started on {}", addr);

    Ok((UdpConnection::new(listener), Arc::new(writers.into())))
}

async fn get_socket(addr: Option<&str>) -> Result<UdpSocket> {
    let socket = match addr {
        None => UdpSocket::bind("0.0.0.0:0").await?,
        Some(a) => UdpSocket::bind(a).await?,
    };
    Ok(socket)
}
