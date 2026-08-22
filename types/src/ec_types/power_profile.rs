use serde::{Deserialize, Serialize};

/// Represents the power profiles
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerProfile {
    Silent,
    #[default]
    Default,
    Performance,
}

impl std::fmt::Display for PowerProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PowerProfile::Silent => "Silent",
            PowerProfile::Default => "Default",
            PowerProfile::Performance => "Performance",
        };
        write!(f, "{}", name)
    }
}
