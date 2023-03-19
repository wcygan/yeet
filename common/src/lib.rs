pub use connection::UdpConnection;
pub use event::{FromServer, ToServer};
mod connection;
mod event;

pub static UDP_PACKET_SIZE: u16 = 512;
pub static DEFAULT_ADDRESS: &'static str = "0.0.0.0:7272";
