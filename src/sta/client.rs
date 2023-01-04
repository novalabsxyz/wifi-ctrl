use super::*;

/// Use a reference counter since ScanResults may be sent to many clients at once
pub type ScanResults = Arc<Vec<ScanResult>>;

#[derive(Debug)]
pub enum SelectResult {
    Success,
    WrongPsk,
    NotFound,
    PendingSelect,
    InvalidNetworkId,
}

use std::fmt;
impl fmt::Display for SelectResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            SelectResult::Success => "success",
            SelectResult::WrongPsk => "wrong_psk",
            SelectResult::NotFound => "network_not_found",
            SelectResult::PendingSelect => "select_already_pending",
            SelectResult::InvalidNetworkId => "invalid_network_id",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub enum Request {
    Status(oneshot::Sender<Result<Status>>),
    Networks(oneshot::Sender<Vec<NetworkResult>>),
    Scan(oneshot::Sender<ScanResults>),
    AddNetwork(oneshot::Sender<usize>),
    SetNetwork(usize, SetNetwork),
    SaveConfig,
    RemoveNetwork(usize),
    SelectNetwork(usize, oneshot::Sender<SelectResult>),
    Shutdown,
}

#[derive(Debug)]
pub enum SetNetwork {
    Ssid(String),
    Psk(String),
}

#[derive(Clone)]
/// Request client wraps the request events, awaiting oneshot channels when appropriate
pub struct RequestClient {
    sender: mpsc::Sender<Request>,
}

impl RequestClient {
    pub fn new(sender: mpsc::Sender<Request>) -> RequestClient {
        RequestClient { sender }
    }

    pub async fn get_scan(&self) -> Result<Arc<Vec<ScanResult>>> {
        let (response, request) = oneshot::channel();
        self.sender.send(Request::Scan(response)).await?;
        Ok(request.await?)
    }

    pub async fn get_networks(&self) -> Result<Vec<NetworkResult>> {
        let (response, request) = oneshot::channel();
        self.sender.send(Request::Networks(response)).await?;
        Ok(request.await?)
    }

    pub async fn get_status(&self) -> Result<Result<Status>> {
        let (response, request) = oneshot::channel();
        self.sender.send(Request::Status(response)).await?;
        Ok(request.await?)
    }

    pub async fn add_network(&self) -> Result<usize> {
        let (response, request) = oneshot::channel();
        self.sender.send(Request::AddNetwork(response)).await?;
        Ok(request.await?)
    }

    pub async fn set_network_psk(&self, network_id: usize, psk: String) -> Result {
        self.sender
            .send(Request::SetNetwork(network_id, SetNetwork::Psk(psk)))
            .await?;
        Ok(())
    }

    pub async fn set_network_ssid(&self, network_id: usize, ssid: String) -> Result {
        self.sender
            .send(Request::SetNetwork(network_id, SetNetwork::Ssid(ssid)))
            .await?;
        Ok(())
    }

    pub async fn save_config(&self) -> Result {
        self.sender.send(Request::SaveConfig).await?;
        Ok(())
    }

    pub async fn remove_network(&self, network_id: usize) -> Result {
        self.sender.send(Request::RemoveNetwork(network_id)).await?;
        Ok(())
    }

    pub async fn select_network(&self, network_id: usize) -> Result<SelectResult> {
        let (response, request) = oneshot::channel();
        self.sender
            .send(Request::SelectNetwork(network_id, response))
            .await?;
        Ok(request.await?)
    }
}

#[derive(Debug, Clone)]
pub enum Broadcast {
    Connected,
    Disconnected,
    NetworkNotFound,
    WrongPsk,
    Ready,
}

/// Channel for broadcasting events. Subscribing to this channel is equivalent to
/// "wpa_ctrl_attach". Can be temporarily silenced using broadcast::Receiver's unsubscribe
pub type BroadcastReceiver = broadcast::Receiver<Broadcast>;
