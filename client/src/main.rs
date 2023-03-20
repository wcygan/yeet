use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use lib_wc::sync::{ShutdownController, ShutdownListener};
use std::future::Future;
use std::io::{self, BufRead, Write};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UdpSocket;
use tokio::select;
mod args;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (mut conn, addr) = init().await?;
    let server_addr = addr.parse()?;
    let name = input_sync("enter your name: ")?;

    // Connect to the server
    let join = ToServer::join(name.clone());
    conn.write::<ToServer>(&join, server_addr).await?;
    match conn.read::<FromServer>().await {
        Ok((FromServer::Ack, src)) => {
            println!("Connection established, {}!", name)
        }
        _ => return Err(anyhow::anyhow!("Unable to connect to the server. Goodbye!")),
    }

    let shutdown = ShutdownController::new();
    let mut listener = shutdown.subscribe();

    select! {
        _ = tokio::signal::ctrl_c() => {
            println!("shutting down!");
            shutdown.shutdown().await;
            println!("done!")
        }
        _ = process(server_addr, &mut conn, &mut listener) => {}
    }

    return Ok(());
}

async fn init() -> Result<(Socket, String)> {
    let args = args::Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    Ok((Socket::new(socket), args.address))
}

fn input_sync(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut s = String::new();
    let _ = io::stdin().lock().read_line(&mut s)?;
    Ok(s.trim().to_owned())
}

async fn process(
    server_addr: SocketAddr,
    conn: &mut Socket,
    shutdown: &mut ShutdownListener,
) -> Result<()> {
    let reader = BufReader::new(tokio::io::stdin());
    let mut lines = reader.lines();
    while !shutdown.is_shutdown() {
        select! {
            _ = shutdown.recv() => {
                conn.write::<ToServer>(&ToServer::Leave, server_addr).await?;
                drop(shutdown);
                return Ok(())
            },
            // TODO: refactor this since it blocks on a thread & causes deadlock
            line = lines.next_line() => {
                let s = line.unwrap().unwrap();
                conn.write::<ToServer>(
                    &ToServer::Message { message: s },
                    server_addr
                );
            },
            msg = conn.read::<FromServer>() => {
                match msg {
                    Ok((FromServer::Message { message }, addr)) => {
                        println!("{}", message)
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
