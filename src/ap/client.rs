use super::*;

#[derive(Debug)]
pub(crate) enum Request {
    Status(oneshot::Sender<Status>),
    Shutdown,
}

impl ShutdownSignal for Request {
    fn is_shutdown(&self) -> bool {
        matches!(self, Request::Shutdown)
    }
}

#[derive(Clone)]
/// Request client wraps the request events, awaiting oneshot channels when appropriate
pub struct RequestClient {
    sender: mpsc::Sender<Request>,
}

impl RequestClient {
    pub(crate) fn new(sender: mpsc::Sender<Request>) -> RequestClient {
        RequestClient { sender }
    }

    async fn send_request(&self, request: Request) -> Result {
        self.sender
            .send(request)
            .await
            .map_err(|_| error::Error::WifiApRequestChannelClosed)?;
        Ok(())
    }

    pub async fn get_status(&self) -> Result<Status> {
        let (response, request) = oneshot::channel();
        self.send_request(Request::Status(response)).await?;
        Ok(request.await?)
    }

    pub async fn shutdown(&self) -> Result {
        self.send_request(Request::Shutdown).await
    }
}

#[derive(Debug, Clone)]
/// Broadcast events, such as a client disconnecting or connecting, may happen at any time.
pub enum Broadcast {
    Ready,
    Connected(String),
    Disconnected(String),
}

/// Channel for broadcasting events.
pub type BroadcastReceiver = broadcast::Receiver<Broadcast>;
