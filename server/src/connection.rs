use crate::stream_drop::CustomListener;
use axum::{extract::connect_info::Connected, serve::IncomingStream};
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct ClientSocket(SocketAddr);

impl std::ops::Deref for ClientSocket {
    type Target = SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Connected<IncomingStream<'a, CustomListener>> for ClientSocket {
    fn connect_info(stream: IncomingStream<'a, CustomListener>) -> Self {
        let remote_addr = *stream.remote_addr();
        Self(remote_addr)
    }
}

// impl Connected<&TcpStream> for ClientConnInfo {
//     fn connect_info(target: &TcpStream) -> Self {
//         let remote_addr = target.peer_addr().unwrap();
//         Self(remote_addr)
//     }
// }
