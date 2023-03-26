use crate::client::Client;
use crate::keyboard_input::recv_from_stdin;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use lib_wc::sync::{ShutdownController, ShutdownListener};
use std::io;
use std::io::{stdin, BufRead, Write};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::select;

mod args;
mod client;
mod keyboard_input;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let shutdown = ShutdownController::new();
    let mut client = Client::new(shutdown.subscribe()).await?;

    select! {
        _ = tokio::signal::ctrl_c() => {
            println!("shutting down");
            shutdown.shutdown().await;
            println!("Done!");
        },
        res = tokio::spawn(async move { client.process().await }) => {
            return res?
        }
    }

    Ok(())
}
