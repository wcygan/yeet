use anyhow::Result;
use clap::Parser;
use tokio::net::UdpSocket;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    let addr = format!("{}:{}", args.address, args.port);
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    socket.send_to(b"hello", addr).await?;
    let mut buf = [0; 512];
    socket.recv_from(&mut buf).await?;

    println!("{}", String::from_utf8_lossy(buf.as_slice()));

    Ok(())
}
