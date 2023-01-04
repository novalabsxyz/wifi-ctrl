use anyhow::anyhow;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::UnixDatagram;
use tokio::sync::{broadcast, mpsc, oneshot};

pub mod ap;
pub mod error;
pub(crate) mod socket_handle;
pub mod sta;

use socket_handle::SocketHandle;
pub type Result<T = ()> = std::result::Result<T, error::Error>;

#[cfg(feature = "logging")]
use log::{debug, error, info, warn};

#[cfg(not(feature = "logging"))]
mod mock_log;
