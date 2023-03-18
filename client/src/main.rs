use anyhow::Result;
use clap::Parser;
use tokio::net::UdpSocket;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    let addr = format!("{}:{}", args.address, args.port);
    let _socket = UdpSocket::bind(addr).await?;
    Ok(())
}
