use super::{error, warn, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::net::UnixDatagram;

#[derive(Serialize, Debug, Clone)]
pub struct ScanResult {
    mac: String,
    frequency: String,
    pub signal: isize,
    flags: String,
    name: String,
}

impl ScanResult {
    pub fn vec_from_str(response: &str) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();
        let split = response.split('\n').skip(1);
        for line in split {
            let mut line_split = line.split_whitespace();
            if let (Some(mac), Some(frequency), Some(signal), Some(flags)) = (
                line_split.next(),
                line_split.next(),
                line_split.next(),
                line_split.next(),
            ) {
                let mut name: Option<String> = None;
                for text in line_split {
                    match &mut name {
                        Some(started) => {
                            started.push(' ');
                            started.push_str(text);
                        }
                        None => {
                            name = Some(text.to_string());
                        }
                    }
                }
                if let Some(name) = name {
                    if let Ok(signal) = isize::from_str(signal) {
                        let scan_result = ScanResult {
                            mac: mac.to_string(),
                            frequency: frequency.to_string(),
                            signal,
                            flags: flags.to_string(),
                            name,
                        };
                        results.push(scan_result);
                    } else {
                        warn!("Invalid string for signal: {signal}");
                    }
                }
            }
        }
        Ok(results)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NetworkResult {
    network_id: usize,
    ssid: String,
    flags: String,
}

impl NetworkResult {
    pub async fn vec_from_str(
        response: &str,
        socket: &mut UnixDatagram,
    ) -> Result<Vec<NetworkResult>> {
        let mut buffer = [0; 256];
        let mut results = Vec::new();
        let split = response.split('\n').skip(1);
        for line in split {
            let mut line_split = line.split_whitespace();
            if let Some(network_id) = line_split.next() {
                let cmd = format!("GET_NETWORK {network_id} ssid");
                let bytes = cmd.into_bytes();
                socket.send(&bytes).await?;
                let n = socket.recv(&mut buffer).await?;
                let ssid = std::str::from_utf8(&buffer[..n])?.trim_matches('\"');
                if let Ok(network_id) = usize::from_str(network_id) {
                    if let Some(flags) = line_split.last() {
                        results.push(NetworkResult {
                            flags: flags.into(),
                            ssid: ssid.into(),
                            network_id,
                        })
                    }
                } else {
                    warn!("Invalid network_id: {network_id}")
                }
            }
        }
        Ok(results)
    }
}

pub type Status = HashMap<String, String>;

pub fn parse_status(response: &str) -> Result<Status> {
    use config::{Config, File, FileFormat};
    let config = Config::builder()
        .add_source(File::from_str(response, FileFormat::Ini))
        .build()
        .map_err(|e| error::Error::ParsingWifiStatus {
            e,
            s: response.into(),
        })?;
    Ok(config.try_deserialize::<HashMap<String, String>>().unwrap())
}
