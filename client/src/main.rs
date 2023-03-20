use crate::keyboard_input::recv_from_stdin;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use lib_wc::sync::{ShutdownController, ShutdownListener};
use std::io;
use std::io::{stdin, BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::select;
use tokio::sync::mpsc;

mod args;
mod keyboard_input;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // TODO: wrap all of this data & the `process` fn inside of a struct :)
    let (mut socket, addr) = init().await?;
    let server_addr: SocketAddr = addr.parse()?;

    let name = input_sync("enter your name: ")?;

    let shutdown = ShutdownController::new();
    let listener = shutdown.subscribe();

    select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown.shutdown().await;
            println!("Done!");
        },
        _ = tokio::spawn(async move { process(name, socket, server_addr, listener).await }) => {}
    }

    Ok(())
}

async fn process(
    name: String,
    mut socket: Socket,
    server_addr: SocketAddr,
    mut sd: ShutdownListener,
) -> Result<()> {
    join(&mut socket, server_addr, name).await?;

    let mut chan = recv_from_stdin();
    while !sd.is_shutdown() {
        select! {
            _ = sd.recv() => {
                break
            },
            line = chan.recv() => {
                if let Some(s) = line {
                    // TODO: handle this with retry?
                    //       `backon` library?
                    let _ = socket.write::<ToServer>(
                        &ToServer::Message { message: s },
                            server_addr
                    ).await;
                }
            }
        }
    }

    Ok(())
}

async fn init() -> Result<(Socket, String)> {
    let args = args::Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    Ok((Socket::new(socket), args.address))
}

async fn join(socket: &mut Socket, server_addr: SocketAddr, name: String) -> Result<()> {
    let join = ToServer::join(name.clone());
    socket.write::<ToServer>(&join, server_addr).await?;
    match socket.read::<FromServer>().await {
        Ok((FromServer::Ack, src)) => {
            println!("Connection established, {}!", name)
        }
        _ => return Err(anyhow::anyhow!("Unable to connect to the server. Goodbye!")),
    }

    Ok(())
}

fn input_sync(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    let _ = stdin().lock().read_line(&mut s)?;
    Ok(s.trim().to_owned())
}
