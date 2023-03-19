use crate::actors::Listener;
use anyhow::Result;
use clap::Parser;
use common::{FromServer, Socket, ToServer};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tub::Pool;

mod actors;
mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let mut listener = Listener::new().await?;
    listener.listen().await;
    Ok(())
}
