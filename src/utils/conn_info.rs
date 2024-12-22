use axum::serve::IncomingStream;

#[derive(Clone, Debug)]
pub struct ClientConnInfo {
    ip: String,
    // port: u16,
}

impl ClientConnInfo {
    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn into_ip(self) -> String {
        self.ip
    }

    // pub fn port(&self) -> u16 {
    //     self.port
    // }
}

impl<'a> axum::extract::connect_info::Connected<IncomingStream<'a>> for ClientConnInfo {
    fn connect_info(target: IncomingStream<'a>) -> Self {
        let remote_addr = target.remote_addr();
        Self {
            ip: remote_addr.ip().to_string(),
            // port: remote_addr.port(),
        }
    }
}
