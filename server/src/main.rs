use anyhow::Result;
use futures::future::join_all;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tub::Pool;

static DEFAULT_PORT: usize = 6565;

type UdpPool = Arc<Pool<UdpSocket>>;

mod client;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let _listener = UdpSocket::bind(format!("0.0.0.0:{}", DEFAULT_PORT)).await?;
    let _pool = create_pool(8).await?;
    Ok(())
}

async fn create_pool(size: usize) -> Result<UdpPool> {
    let futures = (0..size)
        .map(|_| UdpSocket::bind(format!("0.0.0.0:0")))
        .collect::<Vec<_>>();

    let results = join_all(futures).await;

    let sockets = results
        .into_iter()
        .map(|socket| socket.map_err(|e| e.into()))
        .collect::<Result<Vec<_>>>()?;

    Ok(Arc::new(Pool::from_vec(sockets)))
}
