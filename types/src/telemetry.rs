use crate::ec_types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum TelemetryData {
    // todo: improve it
    Startup {
        firmware: String,
        offset: u16,
        cpu: String,
        os: String,
        motherboard: String,
    },

    Status {
        profile: PowerProfile,
        cpu_temp_c: u32,
        sys_temp_c: u32,
        cpu_fan_rpm: u32,
        gpu_fan_rpm: u32,
    },

    Unsupported {
        motherboard: String,
        chip: Option<String>,
    },

    ErrorLog {
        error: String,
    },

    Panic {
        error: String,
    },

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPayload {
    #[serde(with = "crate::hex_u64")]
    pub id: u64,
    pub data: TelemetryData,
}
