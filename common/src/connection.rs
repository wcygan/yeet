use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

pub struct Connection {
    buffer: [u8; 512],
    socket: UdpSocket,
}

impl Connection {
    pub fn new(socket: UdpSocket) -> Self {
        let mut buffer = [0; 512];
        Self { buffer, socket }
    }
}
