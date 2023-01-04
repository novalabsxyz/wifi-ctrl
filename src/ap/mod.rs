#![allow(dead_code)]
use super::*;

mod types;
pub use types::*;

mod client;
pub use client::*;

mod setup;
pub use setup::*;

mod event_socket;
pub use event_socket::*;

const PATH_DEFAULT_SERVER: &str = "/var/run/hostapd/wlan1";

/// Instance that runs the Wifi process
pub struct WifiAp {
    /// Path to the socket
    socket_path: String,
    /// Channel for receiving requests
    request_receiver: mpsc::Receiver<Request>,
    #[allow(unused)]
    /// Channel for broadcasting alerts
    broadcast_sender: broadcast::Sender<Broadcast>,
}

impl WifiAp {
    pub async fn run(self) -> Result {
        info!("Starting Wifi AP process");

        tokio::select!(
            resp = async move {
                // We start up a separate socket for receiving the "unexpected" events that
                // gets forwarded to us via the event_receiver
                let (event_receiver, event_socket) = EventSocket::new().await?;
                let socket_handle = SocketHandle::open(PATH_DEFAULT_SERVER, "mapper_hostapd_sync.sock").await?;
                self.broadcast_sender.send(Broadcast::Ready)?;
                tokio::select!(
                    resp = event_socket.run() => resp,
                    resp = self.run_internal(event_receiver, socket_handle) => resp,
                )
            } => {
                println!("{resp:?}");
                resp
            },
        )
    }

    pub async fn run_internal(
        mut self,
        mut event_receiver: EventReceiver,
        mut socket_handle: SocketHandle<2048>,
    ) -> Result {
        enum EventOrRequest {
            Event(Option<Event>),
            Request(Option<Request>),
        }

        loop {
            let event_or_request = tokio::select!(
                event = event_receiver.recv() => EventOrRequest::Event(event),
                request = self.request_receiver.recv() => EventOrRequest::Request(request),
            );
            match event_or_request {
                EventOrRequest::Event(event) => match event {
                    Some(event) => {
                        Self::handle_event(&mut socket_handle, &self.broadcast_sender, event)
                            .await?
                    }
                    None => return Err(anyhow!("Unexpected close of WiFi AP event channel").into()),
                },
                EventOrRequest::Request(request) => match request {
                    Some(Request::Shutdown) => return Ok(()),
                    Some(request) => Self::handle_request(&mut socket_handle, request).await?,
                    None => {
                        return Err(anyhow!("Unexpected close of WiFi AP requests channel").into())
                    }
                },
            }
        }
    }

    pub async fn handle_event<const N: usize>(
        _socket_handle: &mut SocketHandle<N>,
        broadcast_sender: &broadcast::Sender<Broadcast>,
        event_msg: Event,
    ) -> Result {
        match event_msg {
            Event::ApStaConnected(mac) => {
                if let Err(e) = broadcast_sender.send(client::Broadcast::Connected(mac)) {
                    warn!("error broadcasting: {e}");
                }
            }
            Event::ApStaDisconnected(mac) => {
                if let Err(e) = broadcast_sender.send(client::Broadcast::Disconnected(mac)) {
                    warn!("error broadcasting: {e}");
                }
            }
        };
        Ok(())
    }

    pub async fn handle_request<const N: usize>(
        socket_handle: &mut SocketHandle<N>,
        request: Request,
    ) -> Result {
        debug!("Handling request: {request:?}");
        match request {
            Request::Status(response_channel) => {
                let _n = socket_handle.socket.send(b"STATUS").await?;
                let n = socket_handle.socket.recv(&mut socket_handle.buffer).await?;
                let data_str = std::str::from_utf8(&socket_handle.buffer[..n])?.trim_end();
                let status = Status::from_response(data_str)?;

                if response_channel.send(status).is_err() {
                    error!("Status request response channel closed before response sent");
                }
            }
            Request::Shutdown => (), //shutdown is handled at the scope above
        }
        Ok(())
    }
}
