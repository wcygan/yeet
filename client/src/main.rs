use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use std::io;
use std::io::{BufRead, Write};
use tokio::net::UdpSocket;

mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (mut conn, addr) = init().await?;
    let server_addr = addr.parse()?;
    let _name = get_input("enter your name: ")?;

    loop {
        let s = get_input("> ")?;
        let e = ToServer::Message { message: s };
        conn.write::<ToServer>(&e, server_addr).await?;
        let (value, _) = conn.read::<FromServer>().await?;

        match value {
            FromServer::Message { message } => {
                println!("{}", message)
            }
            FromServer::Ping => {}
        }
    }
}

async fn init() -> Result<(Socket, String)> {
    let args = args::Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    Ok((Socket::new(socket), args.address))
}

fn get_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    let _ = io::stdin().lock().read_line(&mut s)?;
    Ok(s.trim().to_owned())
}
