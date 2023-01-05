use super::*;

pub(crate) struct EventSocket {
    socket_handle: SocketHandle<256>,
    /// Sends messages to client
    sender: mpsc::Sender<Event>,
}

#[derive(Debug)]
pub(crate) enum Event {
    ScanComplete,
    Connected,
    Disconnected,
    NetworkNotFound,
    WrongPsk,
}

pub(crate) type EventReceiver = mpsc::Receiver<Event>;

impl EventSocket {
    pub(crate) async fn new<P>(socket: P) -> Result<(EventReceiver, Self)>
        where
            P: AsRef<std::path::Path> + std::fmt::Debug,{
        let socket_handle =
            SocketHandle::open(socket, "mapper_wpa_ctrl_async.sock").await?;
        let (sender, receiver) = mpsc::channel(32);
        Ok((
            receiver,
            Self {
                socket_handle,
                sender,
            },
        ))
    }

    async fn send_event(&self, event: Event) -> Result {
        self.sender
            .send(event)
            .await
            .map_err(|_| error::Error::WifiStationEventChannelClosed)?;
        Ok(())
    }

    pub(crate) async fn run(mut self) -> Result {
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
                        self.send_event(Event::ScanComplete).await?;
                    }
                    if data_str.contains("CTRL-EVENT-CONNECTED") {
                        self.send_event(Event::Connected).await?;
                    }
                    if data_str.contains("CTRL-EVENT-DISCONNECTED") {
                        self.send_event(Event::Disconnected).await?;
                    }
                    if data_str.contains("CTRL-EVENT-NETWORK-NOT-FOUND") {
                        self.send_event(Event::NetworkNotFound).await?;
                    }
                    if data_str.contains("CTRL-EVENT-SSID-TEMP-DISABLED")
                        && data_str.contains("reason=WRONG_KEY")
                    {
                        self.send_event(Event::WrongPsk).await?;
                    }
                }
                Err(e) => {
                    return Err(error::Error::UnsolicitedIoError(e));
                }
            }
        }
    }
}
