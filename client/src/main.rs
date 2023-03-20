use crate::keyboard_input::recv_from_stdin;
use anyhow::Result;
use clap::Parser;
use common::Socket;
use lib_wc::sync::{ShutdownController, ShutdownListener};
use std::io::{stdin, BufRead, BufReader};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::select;
use tokio::sync::mpsc;

mod args;
mod keyboard_input;

#[tokio::main]
async fn main() -> Result<()> {
    let (mut conn, addr) = init().await?;
    let server_addr: SocketAddr = addr.parse()?;
    let shutdown = ShutdownController::new();
    let listener = shutdown.subscribe();

    select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown.shutdown().await;
            println!("Done!");
        },
        _ = tokio::spawn(async { process(listener).await }) => {}
    }

    Ok(())
}

async fn process(mut sd: ShutdownListener) -> Result<()> {
    let mut chan = recv_from_stdin();
    while !sd.is_shutdown() {
        select! {
            _ = sd.recv() => {
                break
            },
            line = chan.recv() => {
                if let Some(s) = line {
                    println!("{s}")
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
