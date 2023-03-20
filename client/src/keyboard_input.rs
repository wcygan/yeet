use std::io::{stdin, BufRead, BufReader};
use tokio::select;
use tokio::sync::mpsc;

/// Create a channel that streams input from stdin.
///
/// Internally this creates a separate thread to block on stdin.
pub fn recv_from_stdin() -> mpsc::Receiver<String> {
    let (tx, mut rx) = mpsc::channel::<String>(10);
    std::thread::spawn(move || block_on_stdin(tx));
    rx
}

fn block_on_stdin(tx: mpsc::Sender<String>) {
    loop {
        let reader = BufReader::new(stdin());
        let mut lines = reader.lines();
        if let Some(r) = lines.next() {
            if let Ok(s) = r {
                let _ = tx.blocking_send(s);
            }
        }
    }
}
