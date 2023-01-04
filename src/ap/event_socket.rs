use super::*;

pub struct EventSocket {
    socket_handle: SocketHandle<256>,
    /// Sends messages to client
    sender: mpsc::Sender<Event>,
}

#[derive(Debug)]
pub enum Event {
    ApStaConnected(String),
    ApStaDisconnected(String),
}

pub type EventReceiver = mpsc::Receiver<Event>;

impl EventSocket {
    pub async fn new() -> Result<(EventReceiver, Self)> {
        let socket_handle =
            SocketHandle::open(PATH_DEFAULT_SERVER, "mapper_hostapd_async.sock").await?;

        // setup the channel for client requests
        let (sender, receiver) = mpsc::channel(32);
        Ok((
            receiver,
            Self {
                socket_handle,
                sender,
            },
        ))
    }

    pub async fn run(mut self) -> Result {
        let mut attach = self.socket_handle.command(b"ATTACH").await;
        while attach.is_err() {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            attach = self.socket_handle.command(b"ATTACH").await;
        }

        let mut log_level = self.socket_handle.command(b"LOG_LEVEL DEBUG").await;
        while log_level.is_err() {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            log_level = self.socket_handle.command(b"LOG_LEVEL DEBUG").await;
        }
        info!("hostapd event stream registered");

        loop {
            match self
                .socket_handle
                .socket
                .recv(&mut self.socket_handle.buffer)
                .await
            {
                Ok(n) => {
                    let data_str = std::str::from_utf8(&self.socket_handle.buffer[..n])?.trim_end();
                    debug!("hostapd event: {data_str}");
                    if let Some(n) = data_str.find("AP-STA-DISCONNECTED") {
                        let index = n + "AP-STA-DISCONNECTED".len();
                        let mac = &data_str[index..];
                        self.sender
                            .send(Event::ApStaDisconnected(mac.to_string()))
                            .await?;
                    } else if let Some(n) = data_str.find("AP-STA-CONNECTED") {
                        let index = n + "AP-STA-CONNECTED".len();
                        let mac = &data_str[index..];
                        self.sender
                            .send(Event::ApStaConnected(mac.to_string()))
                            .await?;
                    }
                }
                Err(e) => {
                    return Err(error::Error::UnsolicitedIoError(e));
                }
            }
        }
    }
}
