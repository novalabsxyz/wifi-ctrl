use super::*;

pub struct WifiSetup<const C: usize = 32, const B: usize = 32> {
    /// Struct for handling runtime process
    wifi: WifiStation,
    /// Client for making requests
    request_client: RequestClient,
    #[allow(unused)]
    /// Client for receiving alerts
    broadcast_receiver: BroadcastReceiver,
}

impl<const C: usize, const B: usize> WifiSetup<C, B> {
    pub fn new() -> Result<Self> {
        // setup the channel for client requests
        let (sender, request_receiver) = mpsc::channel(C);
        let request_client = RequestClient::new(sender);
        // setup the channel for broadcasts
        let (broadcast_sender, broadcast_receiver) = broadcast::channel(B);

        Ok(Self {
            wifi: WifiStation {
                socket_path: PATH_DEFAULT_SERVER.to_string(),
                request_receiver,
                broadcast_sender,
            },
            request_client,
            broadcast_receiver,
        })
    }

    pub fn set_socket_path(&mut self, path: String) {
        self.wifi.socket_path = path;
    }

    pub fn get_broadcast_receiver(&self) -> BroadcastReceiver {
        self.wifi.broadcast_sender.subscribe()
    }
    pub fn get_request_client(&self) -> RequestClient {
        self.request_client.clone()
    }

    pub fn complete(self) -> WifiStation {
        self.wifi
    }
}
