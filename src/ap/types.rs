use super::{error, Result};
use serde::{de, Deserialize, Serialize};

/// Status of the WiFi Station
#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub state: String,
    pub phy: String,
    pub freq: String,
    pub num_sta_non_erp: String,
    pub num_sta_no_short_slot_time: String,
    pub num_sta_no_short_preamble: String,
    pub olbc: String,
    pub num_sta_ht_no_gf: String,
    pub num_sta_no_ht: String,
    pub num_sta_ht_20_mhz: String,
    pub num_sta_ht40_intolerant: String,
    pub olbc_ht: String,
    pub ht_op_mode: String,
    pub cac_time_seconds: String,
    pub cac_time_left_seconds: String,
    pub channel: String,
    pub secondary_channel: String,
    pub ieee80211n: String,
    pub ieee80211ac: String,
    pub ieee80211ax: String,
    pub beacon_int: String,
    pub dtim_period: String,
    pub ht_caps_info: String,
    pub ht_mcs_bitmask: String,
    pub supported_rates: String,
    pub max_txpower: String,
    pub bss: Vec<String>,
    pub bssid: Vec<String>,
    pub ssid: Vec<String>,
    pub num_sta: Vec<String>,
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

// Configuration of the WiFi station
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bssid: String,
    pub ssid: String,
    #[serde(deserialize_with = "deserialize_enabled_bool")]
    pub wps_state: bool,
    pub wpa: i32,
    pub ket_mgmt: String,
    pub group_cipher: String,
    pub rsn_pairwise_cipher: String,
    pub wpa_pairwise_cipher: String,
}

impl Config {
    pub fn from_response(response: &str) -> Result<Config> {
        use config::{File, FileFormat};
        let config = config::Config::builder()
            .add_source(File::from_str(response, FileFormat::Ini))
            .build()
            .map_err(|e| error::Error::ParsingWifiConfig {
                e,
                s: response.into(),
            })?;

        Ok(config.try_deserialize::<Config>().unwrap())
    }
}

fn deserialize_enabled_bool<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;

    match s {
        "enabled" => Ok(true),
        "disabled" => Ok(false),
        _ => Err(de::Error::unknown_variant(s, &["enabled", "disabled"])),
    }
}
