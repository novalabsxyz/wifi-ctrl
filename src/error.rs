use super::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("start-up aborted")]
    StartupAborted,
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
    #[error("wifi_ctrl::station internal request channel unexpectedly closed")]
    WifiStationRequestChannelClosed,
    #[error("wifi_ctrl::station internal event channel unexpectedly closed")]
    WifiStationEventChannelClosed,
    #[error("wifi_ctrl::ap internal request channel unexpectedly closed")]
    WifiApRequestChannelClosed,
    #[error("wifi_ctrl::ap internal event channel unexpectedly closed")]
    WifiApEventChannelClosed,
    #[error("wifi ap broadcast: {0}")]
    WifiApBroadcast(#[from] broadcast::error::SendError<ap::Broadcast>),
    #[error("wifi::sta broadcast: {0}")]
    WifiStaBroadcast(#[from] broadcast::error::SendError<sta::Broadcast>),
    #[error("wifi sta select result")]
    WifiSelect,
    #[error("timeout opening socket {0}")]
    TimeoutOpeningSocket(String),
    #[error("permission denied opening socket {0}")]
    PermissionDeniedOpeningSocket(String),
}
