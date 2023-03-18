use anyhow::Result;
use std::sync::Arc;
use tokio::net::{ToSocketAddrs, UdpSocket};
use tub::Pool;

#[tokio::main]
async fn main() -> Result<()> {
    let (listener, pool) = get_sockets().await?;

    loop {
        let mut buf = [0; 512];
        let (size, src) = listener.recv_from(&mut buf).await?;
        let pool = pool.clone();
        tokio::spawn(async move {
            let writer = pool.acquire().await;
            writer.send_to(buf.as_slice(), src).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn get_sockets() -> Result<(UdpSocket, Arc<Pool<UdpSocket>>)> {
    let mut writers: Vec<UdpSocket> = vec![];
    for _ in 0..10 {
        writers.push(bind_any().await?)
    }

    let listener = bind_any().await?;
    let addr = listener.local_addr()?;
    println!("Server started on {}", addr);

    Ok((listener, Arc::new(writers.into())))
}

async fn bind_any() -> Result<UdpSocket> {
    Ok(UdpSocket::bind("0.0.0.0:0").await?)
}
