use std::sync::Arc;
use tokio::net::UdpSocket;
use tub::Pool;

struct Server {
    socket: UdpSocket,
    inner: Arc<ServerInner>,
}

struct ServerInner {
    pool: Pool<UdpSocket>,
}
