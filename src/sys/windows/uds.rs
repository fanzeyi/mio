pub use uds_windows::{SocketAddr, UnixStream};

pub(crate) mod stream {
    use std::io;
    use std::path::Path;

    use super::{SocketAddr, UnixStream};

    pub(crate) fn connect(path: &Path) -> io::Result<UnixStream> {
        let stream = UnixStream::connect(path)?;
        stream.set_nonblocking(true)?;
        Ok(stream)
    }

    pub(crate) fn pair() -> io::Result<(UnixStream, UnixStream)> {
        let (left, right) = UnixStream::pair()?;
        left.set_nonblocking(true)?;
        right.set_nonblocking(true)?;
        Ok((left, right))
    }

    pub(crate) fn local_addr(socket: &UnixStream) -> io::Result<SocketAddr> {
        socket.local_addr()
    }

    pub(crate) fn peer_addr(socket: &UnixStream) -> io::Result<SocketAddr> {
        socket.peer_addr()
    }
}

pub(crate) mod listener {
    use std::io;
    use std::os::windows::prelude::{AsRawSocket, FromRawSocket, IntoRawSocket, RawSocket};
    use std::path::Path;

    use super::SocketAddr;

    // TODO(zeyi): maybe not necessary, can be merged into bind/accept below
    #[derive(Debug)]
    pub struct UnixListener {
        inner: uds_windows::UnixListener,
    }

    impl UnixListener {
        fn bind(path: &Path) -> io::Result<Self> {
            let inner = uds_windows::UnixListener::bind(path)?;
            inner.set_nonblocking(true)?;
            Ok(Self::from_std(inner))
        }

        fn from_std(inner: uds_windows::UnixListener) -> Self {
            Self { inner }
        }

        fn accept(&self) -> io::Result<(uds_windows::UnixStream, SocketAddr)> {
            eprintln!("accepting new connections");
            let (conn, addr) = self.inner.accept()?;
            eprintln!("accepted new connections");
            conn.set_nonblocking(true)?;
            Ok((conn, addr))
        }

        fn local_addr(&self) -> io::Result<SocketAddr> {
            self.inner.local_addr()
        }

        pub fn take_error(&self) -> io::Result<Option<io::Error>> {
            self.inner.take_error()
        }
    }

    pub(crate) fn bind(path: &Path) -> io::Result<UnixListener> {
        UnixListener::bind(path)
    }

    pub(crate) fn accept(
        listener: &UnixListener,
    ) -> io::Result<(crate::net::UnixStream, SocketAddr)> {
        listener
            .accept()
            .map(|(conn, addr)| (crate::net::UnixStream::from_std(conn), addr))
    }

    pub(crate) fn local_addr(listener: &UnixListener) -> io::Result<SocketAddr> {
        listener.local_addr()
    }

    #[cfg(windows)]
    impl IntoRawSocket for UnixListener {
        fn into_raw_socket(self) -> RawSocket {
            self.inner.into_raw_socket()
        }
    }

    #[cfg(windows)]
    impl AsRawSocket for UnixListener {
        fn as_raw_socket(&self) -> RawSocket {
            self.inner.as_raw_socket()
        }
    }

    #[cfg(windows)]
    impl FromRawSocket for UnixListener {
        /// Converts a `RawSocket` to a `UnixListener`.
        ///
        /// # Notes
        ///
        /// The caller is responsible for ensuring that the socket is in
        /// non-blocking mode.
        unsafe fn from_raw_socket(socket: RawSocket) -> UnixListener {
            Self::from_std(FromRawSocket::from_raw_socket(socket))
        }
    }
}

pub use self::listener::UnixListener;
