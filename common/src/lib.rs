pub use event::{FromServer, ToServer};
mod event;

pub static UDP_PACKET_SIZE: u16 = 512;
pub static DEFAULT_ADDRESS: &str = "0.0.0.0:7272";
