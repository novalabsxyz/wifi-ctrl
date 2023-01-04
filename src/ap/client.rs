use super::*;

#[derive(Debug)]
pub enum Request {
    Status(oneshot::Sender<Status>),
    Shutdown,
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

    pub async fn get_status(&self) -> Result<Status> {
        let (response, request) = oneshot::channel();
        self.sender.send(Request::Status(response)).await?;
        Ok(request.await?)
    }
}

#[derive(Debug, Clone)]
pub enum Broadcast {
    Ready,
    Connected(String),
    Disconnected(String),
}

/// Channel for broadcasting events. Subscribing to this channel is equivalent to
/// "wpa_ctrl_attach". Can be temporarily silenced using broadcast::Receiver's unsubscribe
pub type BroadcastReceiver = broadcast::Receiver<Broadcast>;
