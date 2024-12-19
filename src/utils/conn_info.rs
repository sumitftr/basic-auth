use axum::serve::IncomingStream;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct ClientConnInfo {
    remote: SocketAddr,
}

impl ClientConnInfo {
    pub fn ip(&self) -> String {
        self.remote.ip().to_string()
    }
}

impl<'a> axum::extract::connect_info::Connected<IncomingStream<'a>> for ClientConnInfo {
    fn connect_info(target: IncomingStream<'a>) -> Self {
        Self {
            remote: target.remote_addr(),
        }
    }
}
