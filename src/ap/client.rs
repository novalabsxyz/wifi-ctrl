use super::*;

#[derive(Debug)]
pub(crate) enum Request {
    Status(oneshot::Sender<Result<Status>>),
    Config(oneshot::Sender<Result<Config>>),
    Enable(oneshot::Sender<Result>),
    Disable(oneshot::Sender<Result>),
    SetValue(String, String, oneshot::Sender<Result>),
    Shutdown,
}

impl ShutdownSignal for Request {
    fn is_shutdown(&self) -> bool {
        matches!(self, Request::Shutdown)
    }
    fn inform_of_shutdown(self) {
        match self {
            Request::Status(response) => {
                let _ = response.send(Err(error::Error::StartupAborted));
            }
            Request::Config(response) => {
                let _ = response.send(Err(error::Error::StartupAborted));
            }
            Request::Enable(response) | Request::Disable(response) => {
                let _ = response.send(Err(error::Error::StartupAborted));
            }
            Request::SetValue(_, _, response) => {
                let _ = response.send(Err(error::Error::StartupAborted));
            }
            Request::Shutdown => {}
        }
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
        request.await?
    }

    pub async fn get_config(&self) -> Result<Config> {
        let (response, request) = oneshot::channel();
        self.send_request(Request::Config(response)).await?;
        request.await?
    }

    pub async fn enable(&self) -> Result {
        let (response, request) = oneshot::channel();
        self.send_request(Request::Enable(response)).await?;
        request.await?
    }

    pub async fn disable(&self) -> Result {
        let (response, request) = oneshot::channel();
        self.send_request(Request::Disable(response)).await?;
        request.await?
    }

    pub async fn set_value(&self, key: &str, value: &str) -> Result {
        let (response, request) = oneshot::channel();
        self.send_request(Request::SetValue(key.into(), value.into(), response))
            .await?;
        request.await?
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
