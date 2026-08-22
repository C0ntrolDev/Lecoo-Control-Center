use serde::{Deserialize, Serialize};

/// Represents the keyboard backlight brightness levels
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum KeyboardBacklightLevel { // TODO: not every revision supports all levels!!!!! fix it later
    #[default]
    Off,
    Low,
    Medium,
    High,
    Custom(u8),
}

impl std::fmt::Display for KeyboardBacklightLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardBacklightLevel::Off => write!(f, "Off"),
            KeyboardBacklightLevel::Low => write!(f, "Low"),
            KeyboardBacklightLevel::Medium => write!(f, "Medium"),
            KeyboardBacklightLevel::High => write!(f, "High"),
            KeyboardBacklightLevel::Custom(u) => write!(f, "Custom: {u}/255"),
        }
    }
}
