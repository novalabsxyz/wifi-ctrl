use super::*;

mod types;
pub use types::*;

mod client;
pub use client::*;

mod setup;
pub use setup::*;

mod event_socket;
pub use event_socket::*;

const PATH_DEFAULT_SERVER: &str = "/var/run/wpa_supplicant/wlan2";

/// Instance that runs the Wifi process
pub struct WifiStation {
    /// Path to the socket
    socket_path: String,
    /// Channel for receiving requests
    request_receiver: mpsc::Receiver<Request>,
    #[allow(unused)]
    /// Channel for broadcasting alerts
    broadcast_sender: broadcast::Sender<Broadcast>,
}

impl WifiStation {
    pub async fn run(self) -> Result {
        info!("Starting Wifi Station process");

        tokio::select!(
            resp = async move {
                let socket_handle =
                    SocketHandle::open(&self.socket_path, "mapper_wpa_ctrl_sync.sock").await?;
                // We start up a separate socket for receiving the "unexpected" events that
                // gets forwarded to us via the unsolicited_receiver
                let (unsolicited_receiver, unsolicited) = EventSocket::new().await?;
                self.broadcast_sender.send(Broadcast::Ready)?;
                tokio::select!(
                    resp = unsolicited.run() => resp,
                    resp = self.run_internal(unsolicited_receiver, socket_handle) => resp,
                )
            } => resp,
        )
    }

    pub async fn run_internal(
        mut self,
        mut unsolicited_receiver: EventReceiver,
        mut socket_handle: SocketHandle<10240>,
    ) -> Result {
        // We will collect scan requests and batch respond to them when results are ready
        let mut scan_requests = Vec::new();
        let mut select_request = None;
        loop {
            enum EventOrRequest {
                Event(Option<Event>),
                Request(Option<Request>),
            }

            let event_or_request = tokio::select!(
                unsolicited_msg = unsolicited_receiver.recv() => {
                    EventOrRequest::Event(unsolicited_msg)
                },
                request = self.request_receiver.recv() => {
                    EventOrRequest::Request(request)
                },
            );

            match event_or_request {
                EventOrRequest::Event(event) => match event {
                    Some(unsolicited_msg) => {
                        debug!("Unsolicited event: {unsolicited_msg:?}");
                        Self::handle_event(
                            &mut socket_handle,
                            unsolicited_msg,
                            &mut scan_requests,
                            &mut select_request,
                            &mut self.broadcast_sender,
                        )
                        .await?
                    }
                    None => {
                        return Err(anyhow!(
                            "Unexpected close of WiFi Sta unsolicited event channel"
                        )
                        .into())
                    }
                },
                EventOrRequest::Request(request) => match request {
                    Some(Request::Shutdown) => {
                        return Ok(());
                    }
                    Some(request) => {
                        Self::handle_request(
                            &mut socket_handle,
                            request,
                            &mut scan_requests,
                            &mut select_request,
                        )
                        .await?;
                    }
                    None => {
                        return Err(anyhow!("Unexpected close of WiFi Sta requests channel").into())
                    }
                },
            }
        }
    }

    pub async fn handle_event<const N: usize>(
        socket_handle: &mut SocketHandle<N>,
        event: Event,
        scan_requests: &mut Vec<oneshot::Sender<Arc<Vec<ScanResult>>>>,
        select_request: &mut Option<oneshot::Sender<SelectResult>>,
        broadcast_sender: &mut broadcast::Sender<Broadcast>,
    ) -> Result {
        match event {
            Event::ScanComplete => {
                let _n = socket_handle.socket.send(b"SCAN_RESULTS").await?;
                let n = socket_handle.socket.recv(&mut socket_handle.buffer).await?;
                let data_str = std::str::from_utf8(&socket_handle.buffer[..n])?;
                let mut scan_results = ScanResult::vec_from_str(data_str)?;
                scan_results.sort_by(|a, b| a.signal.cmp(&b.signal));

                let results = Arc::new(scan_results);
                while let Some(scan_request) = scan_requests.pop() {
                    if scan_request.send(results.clone()).is_err() {
                        error!("Scan request response channel closed before response sent");
                    }
                }
            }
            Event::Connected => {
                broadcast_sender.send(Broadcast::Connected)?;
                if let Some(sender) = select_request.take() {
                    sender
                        .send(SelectResult::Success)
                        .map_err(|_| error::Error::WifiSelect)?;
                }
            }
            Event::Disconnected => {
                broadcast_sender.send(Broadcast::Disconnected)?;
            }
            Event::NetworkNotFound => {
                broadcast_sender.send(Broadcast::NetworkNotFound)?;
                if let Some(sender) = select_request.take() {
                    sender
                        .send(SelectResult::NotFound)
                        .map_err(|_| error::Error::WifiSelect)?;
                }
            }
            Event::WrongPsk => {
                broadcast_sender.send(Broadcast::WrongPsk)?;
                if let Some(sender) = select_request.take() {
                    sender
                        .send(SelectResult::WrongPsk)
                        .map_err(|_| error::Error::WifiSelect)?;
                }
            }
        }
        Ok(())
    }

    pub async fn handle_request<const N: usize>(
        socket_handle: &mut SocketHandle<N>,
        request: Request,
        scan_requests: &mut Vec<oneshot::Sender<Arc<Vec<ScanResult>>>>,
        select_request: &mut Option<oneshot::Sender<SelectResult>>,
    ) -> Result {
        debug!("Handling request: {request:?}");
        match request {
            Request::Scan(response_channel) => {
                scan_requests.push(response_channel);
                if let Err(e) = socket_handle.command(b"SCAN").await {
                    debug!("Error while requesting SCAN: {e}");
                }
            }
            Request::Networks(response_channel) => {
                let _n = socket_handle.socket.send(b"LIST_NETWORKS").await?;
                let n = socket_handle.socket.recv(&mut socket_handle.buffer).await?;
                let data_str = std::str::from_utf8(&socket_handle.buffer[..n])?.trim_end();
                let network_list =
                    NetworkResult::vec_from_str(data_str, &mut socket_handle.socket).await?;
                if response_channel.send(network_list).is_err() {
                    error!("Scan request response channel closed before response sent");
                }
            }
            Request::Status(response_channel) => {
                let _n = socket_handle.socket.send(b"STATUS").await?;
                let n = socket_handle.socket.recv(&mut socket_handle.buffer).await?;
                let data_str = std::str::from_utf8(&socket_handle.buffer[..n])?.trim_end();
                let status = parse_status(data_str);
                if response_channel.send(status).is_err() {
                    error!("Scan request response channel closed before response sent");
                }
            }
            Request::AddNetwork(response_channel) => {
                let _n = socket_handle.socket.send(b"ADD_NETWORK").await?;
                let n = socket_handle.socket.recv(&mut socket_handle.buffer).await?;
                let data_str = std::str::from_utf8(&socket_handle.buffer[..n])?.trim_end();
                let network_id = usize::from_str(data_str)?;
                if response_channel.send(network_id).is_err() {
                    error!("Scan request response channel closed before response sent");
                } else {
                    debug!("wpa_ctrl created network {network_id}");
                }
            }
            Request::SetNetwork(id, param) => {
                let cmd = format!(
                    "SET_NETWORK {id} {}",
                    match param {
                        SetNetwork::Ssid(ssid) => format!("ssid \"{ssid}\""),
                        SetNetwork::Psk(psk) => format!("psk \"{psk}\""),
                    }
                );
                debug!("wpa_ctrl \"{cmd}\"");
                let bytes = cmd.into_bytes();
                if let Err(e) = socket_handle.command(&bytes).await {
                    warn!("Error while setting network parameter: {e}");
                }
            }
            Request::SaveConfig => {
                if let Err(e) = socket_handle.command(b"SAVE_CONFIG").await {
                    warn!("Error while saving config: {e}");
                }
                debug!("wpa_ctrl config saved");
            }
            Request::RemoveNetwork(id) => {
                let cmd = format!("REMOVE_NETWORK {id}");
                let bytes = cmd.into_bytes();
                if let Err(e) = socket_handle.command(&bytes).await {
                    warn!("Error while removing network {id}: {e}");
                }
                debug!("wpa_ctrl removed network {id}");
            }
            Request::SelectNetwork(id, response_sender) => {
                let response_sender = match select_request {
                    None => {
                        let cmd = format!("SELECT_NETWORK {id}");
                        let bytes = cmd.into_bytes();
                        if let Err(e) = socket_handle.command(&bytes).await {
                            warn!("Error while selecting network {id}: {e}");
                            response_sender
                                .send(SelectResult::InvalidNetworkId)
                                .map_err(|_| error::Error::WifiSelect)?;
                            None
                        } else {
                            debug!("wpa_ctrl selected network {id}");
                            Some(response_sender)
                        }
                    }
                    Some(_) => {
                        warn!("Select request already pending! Dropping this one.");
                        response_sender
                            .send(SelectResult::PendingSelect)
                            .map_err(|_| error::Error::WifiSelect)?;
                        debug!("wpa_ctrl removed network {id}");
                        None
                    }
                };
                if let Some(response_sender) = response_sender {
                    *select_request = Some(response_sender);
                }
            }
            Request::Shutdown => (), //shutdown is handled at the scope above
        }
        Ok(())
    }
}
