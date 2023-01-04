use super::*;

pub struct EventSocket {
    socket_handle: SocketHandle<256>,
    /// Sends messages to client
    sender: mpsc::Sender<Event>,
}

#[derive(Debug)]
pub enum Event {
    ScanComplete,
    Connected,
    Disconnected,
    NetworkNotFound,
    WrongPsk,
}

pub type EventReceiver = mpsc::Receiver<Event>;

impl EventSocket {
    pub async fn new() -> Result<(EventReceiver, Self)> {
        let socket_handle =
            SocketHandle::open(PATH_DEFAULT_SERVER, "mapper_wpa_ctrl_async.sock").await?;
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
        info!("wpa_ctrl attempting attach");

        self.socket_handle.socket.send(b"ATTACH").await?;
        loop {
            match self
                .socket_handle
                .socket
                .recv(&mut self.socket_handle.buffer)
                .await
            {
                Ok(n) => {
                    let data_str = std::str::from_utf8(&self.socket_handle.buffer[..n])?.trim_end();
                    debug!("wpa_ctrl event: {data_str}");
                    if data_str.ends_with("CTRL-EVENT-SCAN-RESULTS") {
                        self.sender.send(Event::ScanComplete).await?;
                    }
                    if data_str.contains("CTRL-EVENT-CONNECTED") {
                        self.sender.send(Event::Connected).await?;
                    }
                    if data_str.contains("CTRL-EVENT-DISCONNECTED") {
                        self.sender.send(Event::Disconnected).await?;
                    }
                    if data_str.contains("CTRL-EVENT-NETWORK-NOT-FOUND") {
                        self.sender.send(Event::NetworkNotFound).await?;
                    }
                    if data_str.contains("CTRL-EVENT-SSID-TEMP-DISABLED")
                        && data_str.contains("reason=WRONG_KEY")
                    {
                        self.sender.send(Event::WrongPsk).await?;
                    }
                }
                Err(e) => {
                    return Err(error::Error::UnsolicitedIoError(e));
                }
            }
        }
    }
}
