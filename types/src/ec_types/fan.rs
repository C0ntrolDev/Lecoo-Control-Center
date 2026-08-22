use serde::{Deserialize, Serialize};

/// Represents the fan control mode
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum FanMode {
    #[default]
    Auto, // Controlled by EC thermal tables
    Full,       // 100% speed override
    Turbo,      // Turbo mode (without safety)
    Custom(u8), // Custom PWM duty cycle
}

/// Identifies the specific fan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum FanIndex {
    Cpu,
    Gpu, // not actually gpu, actually the cpu 2nd fan
}
