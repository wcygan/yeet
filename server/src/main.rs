use crate::actors::Listener;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use lib_wc::sync::ShutdownController;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::select;
use tub::Pool;

mod actors;
mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let shutdown = ShutdownController::new();
    let mut listener = Listener::new(&shutdown).await?;

    select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown.shutdown().await
        }
        _ = tokio::spawn(async move { listener.listen().await }) => {}
    }

    Ok(())
}
