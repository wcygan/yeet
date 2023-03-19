use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use std::future::Future;
use std::io::{self, BufRead, Write};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (mut conn, addr) = init().await?;
    let server_addr = addr.parse()?;
    let name = input("enter your name: ")?;

    // Connect to the server
    let join = ToServer::join(name.clone());
    conn.write::<ToServer>(&join, server_addr).await?;
    match conn.read::<FromServer>().await {
        Ok((FromServer::Ack, src)) => {
            println!("Connection established, {}!", name)
        }
        _ => return Err(anyhow::anyhow!("Unable to connect to the server. Goodbye!")),
    }

    loop {
        // TODO: select on keyboard input and UDP recv

        let message = ToServer::message(input("> ")?);
        conn.write::<ToServer>(&message, server_addr).await?;
        let (value, _) = conn.read::<FromServer>().await?;

        match value {
            FromServer::Message { message } => {
                println!("{}", message)
            }
            _ => {}
        }
    }
}

async fn init() -> Result<(Socket, String)> {
    let args = args::Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    Ok((Socket::new(socket), args.address))
}

fn input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    let _ = io::stdin().lock().read_line(&mut s)?;
    Ok(s.trim().to_owned())
}
