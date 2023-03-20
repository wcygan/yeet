use crate::keyboard_input::recv_from_stdin;
use lib_wc::sync::{ShutdownController, ShutdownListener};
use std::io::{stdin, BufRead, BufReader};
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;

mod keyboard_input;

#[tokio::main]
async fn main() {
    let shutdown = ShutdownController::new();
    let listener = shutdown.subscribe();

    select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown.shutdown().await;
            println!("Done!");
        },
        _ = tokio::spawn(async { process(listener).await }) => {}
    }
}

async fn process(mut sd: ShutdownListener) {
    let mut chan = recv_from_stdin();
    while !sd.is_shutdown() {
        select! {
            _ = sd.recv() => {
                return;
            },
            line = chan.recv() => {
                if let Some(s) = line {
                    println!("{s}")
                }
            }
        }
    }
}
