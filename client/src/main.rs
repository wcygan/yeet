use anyhow::Result;
use clap::builder::Str;
use clap::Parser;
use common::ToServer;
use tokio::net::UdpSocket;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (socket, addr) = init().await?;

    let e = ToServer::Join {
        name: String::from("Hello"),
    };

    // Send "hello" & read the response into buffer
    let mut buffer = [0; 512];
    socket.send_to(b"hello", addr).await?;
    socket.recv_from(&mut buffer).await?;

    println!("{}", String::from_utf8_lossy(buffer.as_slice()));
    Ok(())
}

async fn init() -> Result<(UdpSocket, String)> {
    let args = args::Args::parse();
    let addr = format!("{}:{}", args.address, args.port);
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    Ok((socket, addr))
}
