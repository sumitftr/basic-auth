use tokio::net::TcpListener;

pub struct CustomStream(tokio::net::TcpStream);

impl tokio::io::AsyncRead for CustomStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for CustomStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl Drop for CustomStream {
    fn drop(&mut self) {
        let socket_addr = match self.0.local_addr() {
            Ok(addr) => addr,
            Err(_) => return, // might cause problem
        };

        tokio::spawn(async move {
            let db = database::Db::new().await;
            if db.remove_oauth_creds(&socket_addr).is_none() {
                let _ = db.remove_applicant(socket_addr).await;
            }
        });
    }
}

pub struct CustomListener(TcpListener);

impl axum::serve::Listener for CustomListener {
    type Io = CustomStream;
    type Addr = std::net::SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (io, addr) = self.0.accept().await.unwrap();
        (CustomStream(io), addr)
    }

    fn local_addr(&self) -> tokio::io::Result<Self::Addr> {
        self.0.local_addr()
    }
}

impl From<TcpListener> for CustomListener {
    fn from(value: TcpListener) -> Self {
        Self(value)
    }
}
