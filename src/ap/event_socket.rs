use super::*;

pub(crate) struct EventSocket {
    socket_handle: SocketHandle<256>,
    attach_options: Vec<String>,
    /// Sends messages to client
    sender: mpsc::Sender<Event>,
}

#[derive(Debug)]
pub(crate) enum Event {
    ApStaConnected(String),
    ApStaDisconnected(String),
    Unknown(String),
}

pub(crate) type EventReceiver = mpsc::Receiver<Event>;

impl EventSocket {
    pub(crate) async fn new<P>(
        socket: P,
        request_receiver: &mut mpsc::Receiver<Request>,
        attach_options: &Vec<String>,
    ) -> Result<(EventReceiver, Vec<Request>, Self)>
    where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let (socket_handle, deferred_requests) =
            SocketHandle::open(socket, "hostapd_async.sock", request_receiver).await?;

        // setup the channel for client requests
        let (sender, receiver) = mpsc::channel(32);
        Ok((
            receiver,
            deferred_requests,
            Self {
                socket_handle,
                sender,
                attach_options: attach_options.clone(),
            },
        ))
    }

    async fn send_event(&self, event: Event) -> Result {
        self.sender
            .send(event)
            .await
            .map_err(|_| error::Error::WifiApEventChannelClosed)?;
        Ok(())
    }

    pub(crate) async fn run(mut self) -> Result {
        info!("Run!");
        let mut command = "ATTACH".to_string();
        for o in &self.attach_options {
            command.push_str(" ");
            command.push_str(o);
        }
        let mut attach = self.socket_handle.command(command.as_bytes()).await;
        while attach.is_err() {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            attach = self.socket_handle.command(command.as_bytes()).await;
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
                    if let Some(n) = data_str.find("AP-STA-DISCONNECTED") {
                        let index = n + "AP-STA-DISCONNECTED".len();
                        let mac = &data_str[index..];
                        self.send_event(Event::ApStaDisconnected(mac.to_string()))
                            .await?;
                    } else if let Some(n) = data_str.find("AP-STA-CONNECTED") {
                        let index = n + "AP-STA-CONNECTED".len();
                        let mac = &data_str[index..];
                        self.send_event(Event::ApStaConnected(mac.to_string()))
                            .await?;
                    } else {
                        self.send_event(Event::Unknown(data_str.to_string()))
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
