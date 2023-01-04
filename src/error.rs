use super::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("error parsing wifi status {e}: \n{s}")]
    ParsingWifiStatus { e: config::ConfigError, s: String },
    #[error("unexpected wifi ap response: {0}")]
    UnexpectedWifiApRepsonse(String),
    #[error("timeout waiting for response")]
    Timeout,
    #[error("did not write all bytes {0}/{1}")]
    DidNotWriteAllBytes(usize, usize),
    #[error("error parsing int: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("utf8 error: {0}")]
    Utf8Parse(#[from] std::str::Utf8Error),
    #[error("recv error: {0}")]
    Recv(#[from] oneshot::error::RecvError),
    #[error("unsolicited socket io error: {0}")]
    UnsolicitedIoError(std::io::Error),
    #[error("wifi station request: {0}")]
    WifiStationRequest(#[from] mpsc::error::SendError<sta::Request>),
    #[error("wifi station event msg: {0}")]
    WifiStationEventMsg(#[from] mpsc::error::SendError<sta::Event>),
    #[error("wifi access point request: {0}")]
    WifiApRequest(#[from] mpsc::error::SendError<ap::Request>),
    #[error("wifi access point event msg: {0}")]
    WifiApEventMsg(#[from] mpsc::error::SendError<ap::Event>),
    #[error("wifi ap broadcast: {0}")]
    WifiApBroadcast(#[from] broadcast::error::SendError<ap::Broadcast>),
    #[error("wifi::sta broadcast: {0}")]
    WifiStaBroadcast(#[from] broadcast::error::SendError<sta::Broadcast>),
    #[error("wifi sta select result")]
    WifiSelect,
    #[error("timeout opening socket {0}")]
    TimeoutOpeningSocket(String),
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}
