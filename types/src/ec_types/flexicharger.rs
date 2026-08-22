use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChargeRange {
    pub min: u8,
    pub max: u8,
}

/// Fixed vocabulary. A new board adds a backend in the daemon, never a variant here.
/// bincode encodes the variant index: append only.
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(tag = "t", content = "c")]
pub enum ChargeIntent {
    #[default]
    Full,
    /// None = thresholds are chosen by firmware (N161A).
    Preserve(Option<ChargeRange>),
    Freeze,
}

/// Represents battery charge limit profiles (FlexiCharger)
/// todo: legacy, remove, make it as ChargePreset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargeLimit {
    FullCapacity,    // 100%
    HighCapacity,    // 95%
    Balanced,        // 80%
    MaximumLifespan, // 60%
    DeskMode,        // 40%
}
impl ChargeLimit {
    pub fn as_percent(&self) -> (u8, u8) {
        match self {
            ChargeLimit::FullCapacity => (0, 0),
            ChargeLimit::HighCapacity => (90, 95),
            ChargeLimit::Balanced => (70, 80),
            ChargeLimit::MaximumLifespan => (55, 60),
            ChargeLimit::DeskMode => (40, 50),
        }
    }

    pub fn from_predefined(min: u8, max: u8) -> Option<Self> {
        if min > max {
            return None;
        }
        match (min, max) {
            (0, 0) => Some(Self::FullCapacity),
            (90, 95) => Some(Self::HighCapacity),
            (70, 80) => Some(Self::Balanced),
            (55, 60) => Some(Self::MaximumLifespan),
            (40, 50) => Some(Self::DeskMode),
            _ => None,
        }
    }
}
