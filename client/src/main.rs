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
    join(&mut socket, server_addr, name, &mut sd).await?;

    let mut chan = recv_from_stdin();
    while !sd.is_shutdown() {
        select! {
            _ = sd.recv() => {
                leave(&mut socket, server_addr).await?;
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
            },
            msg = socket.read::<FromServer>() => {
                match msg {
                    Ok((FromServer::Message { message }, addr)) => {
                        println!("{}", message)
                    },
                    Ok((FromServer::Shutdown, addr)) => {
                        println!("the server told us to shutdown!");
                        return Ok(())
                    },
                    Ok((FromServer::Ping, addr)) => {
                        println!("ping")
                    },
                    Ok((FromServer::Ack, addr)) => {
                        println!("ack")
                    }
                    _ => {}
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

async fn join(
    socket: &mut Socket,
    server_addr: SocketAddr,
    name: String,
    shutdown: &mut ShutdownListener,
) -> Result<()> {
    let join = ToServer::join(name.clone());
    socket.write::<ToServer>(&join, server_addr).await?;

    let wait_for_response = tokio::time::timeout(Duration::from_secs(1), async move {
        match socket.read::<FromServer>().await {
            Ok((FromServer::Ack, src)) => {
                println!("Connection established, {}!", name);
                return Ok::<(), anyhow::Error>(());
            }
            _ => return Err(anyhow::anyhow!("Unable to connect to the server. Goodbye!")),
        }
    });

    select! {
        _ = shutdown.recv() => {
            println!("Shutting down")
        }
        res = wait_for_response => {
            if let Err(e) = res {
                println!("Timeout expired");
                return Err(e.into())
            }
        }
    }

    Ok(())
}

async fn leave(socket: &mut Socket, server_addr: SocketAddr) -> Result<()> {
    let _ = socket
        .write::<ToServer>(&ToServer::Leave, server_addr)
        .await;
    Ok(())
}

fn input_sync(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    let _ = stdin().lock().read_line(&mut s)?;
    Ok(s.trim().to_owned())
}
