use crate::keyboard_input::recv_from_stdin;
use lib_wc::sync::{ShutdownController, ShutdownListener};
use std::io::{stdin, BufRead, BufReader};
use tokio::select;
use tokio::sync::mpsc;

mod keyboard_input;

#[tokio::main]
async fn main() {
    let shutdown = ShutdownController::new();
    select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown.shutdown().await;
        },
        _ = process(shutdown.subscribe()) => {}
    }
}

async fn process(mut listener: ShutdownListener) {
    let mut chan = recv_from_stdin();
    while !listener.is_shutdown() {
        select! {
            _ = listener.recv() => {
                return
            },
            line = chan.recv() => {
                    match line {
                        Some(s) => println!("{s}"),
                        None => {}
                    }
            }
        }
    }
}
