use anyhow::Result;
use bincode::{deserialize, serialize};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::net::UdpSocket;

pub struct UdpConnection {
    _buffer: [u8; 512],
    _socket: UdpSocket,
}

impl UdpConnection {
    pub fn new(socket: UdpSocket) -> Self {
        let buffer = [0; 512];
        Self {
            _buffer: buffer,
            _socket: socket,
        }
    }

    pub async fn write<T: Serialize>(&mut self, _value: &T) -> Result<()> {
        todo!()
    }

    pub async fn read<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        todo!()
    }
}
