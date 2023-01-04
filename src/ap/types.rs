use super::{error, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    state: String,
    phy: String,
    freq: String,
    num_sta_non_erp: String,
    num_sta_no_short_slot_time: String,
    num_sta_no_short_preamble: String,
    olbc: String,
    num_sta_ht_no_gf: String,
    num_sta_no_ht: String,
    num_sta_ht_20_mhz: String,
    num_sta_ht40_intolerant: String,
    olbc_ht: String,
    ht_op_mode: String,
    cac_time_seconds: String,
    cac_time_left_seconds: String,
    channel: String,
    secondary_channel: String,
    ieee80211n: String,
    ieee80211ac: String,
    ieee80211ax: String,
    beacon_int: String,
    dtim_period: String,
    ht_caps_info: String,
    ht_mcs_bitmask: String,
    supported_rates: String,
    max_txpower: String,
    bss: Vec<String>,
    bssid: Vec<String>,
    ssid: Vec<String>,
    num_sta: Vec<String>,
}

impl Status {
    pub fn from_response(response: &str) -> Result<Status> {
        use config::{Config, File, FileFormat};
        let config = Config::builder()
            .add_source(File::from_str(response, FileFormat::Ini))
            .build()
            .map_err(|e| error::Error::ParsingWifiStatus {
                e,
                s: response.into(),
            })?;

        Ok(config.try_deserialize::<Status>().unwrap())
    }
}
