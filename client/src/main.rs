use anyhow::Result;
use clap::Parser;
use common::{FromServer, ToServer, UdpConnection};
use tokio::net::UdpSocket;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (mut conn, addr) = init().await?;
    let server_addr = addr.parse()?;

    let e = ToServer::Message {
        message: String::from("Hello"),
    };

    conn.write::<ToServer>(&e, server_addr).await?;

    let (t, _) = conn.read::<FromServer>().await?;
    println!("{:?}", t);

    Ok(())
}

async fn init() -> Result<(UdpConnection, String)> {
    let args = args::Args::parse();
    let addr = format!("{}:{}", args.address, args.port);
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    Ok((UdpConnection::new(socket), addr))
}
