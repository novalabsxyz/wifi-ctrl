use super::*;

pub struct SocketHandle<const N: usize> {
    #[allow(unused)]
    /// Temporary directory for socket. If it drops, socket breaks.
    tmp_dir: tempfile::TempDir,
    /// Socket for synchronous messages
    pub socket: UnixDatagram,
    pub buffer: [u8; N],
}

impl<const N: usize> SocketHandle<N> {
    pub async fn open<P>(path: P, label: &str) -> Result<Self>
    where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let tmp_dir = tempfile::tempdir()?;
        let connect_from = tmp_dir.path().join(label);
        let socket = UnixDatagram::bind(connect_from)?;
        let socket_debug = format!("{path:?}");
        // loop around waiting for the socket for up to 5 minutes
        let socket = tokio::select!(
            resp = async move  {
                while socket.connect(&path).is_err() {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                Ok(socket)
            } => resp,
            _ = async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(60*5)).await;
            } => Err(error::Error::TimeoutOpeningSocket(socket_debug)),
        )?;

        Ok(Self {
            tmp_dir,
            socket,
            buffer: [0; N],
        })
    }

    pub async fn command(&mut self, cmd: &[u8]) -> Result {
        let n = self.socket.send(cmd).await?;
        if n != cmd.len() {
            return Err(error::Error::DidNotWriteAllBytes(n, cmd.len()));
        }
        self.expect_ok_with_default_timeout().await
    }

    async fn expect_ok(&mut self) -> Result {
        match self.socket.recv(&mut self.buffer).await {
            Ok(n) => {
                let data_str = std::str::from_utf8(&self.buffer[..n])?.trim_end();
                if data_str.trim() == "OK" {
                    Ok(())
                } else {
                    Err(error::Error::UnexpectedWifiApRepsonse(data_str.into()))
                }
            }
            Err(e) => Err(error::Error::UnsolicitedIoError(e)),
        }
    }

    async fn expect_ok_with_default_timeout(&mut self) -> Result {
        self.expect_ok_with_timeout(tokio::time::Duration::from_secs(1))
            .await
    }

    pub async fn expect_ok_with_timeout(&mut self, timeout: tokio::time::Duration) -> Result {
        tokio::select!(
            resp = self.expect_ok() => resp,
            _ =
                tokio::time::sleep(timeout) => Err(error::Error::Timeout)
        )
    }
}
