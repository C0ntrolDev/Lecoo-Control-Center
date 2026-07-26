use std::hash::{DefaultHasher, Hash, Hasher};

use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum DaemonCommand {
    RestoreDefaults,
    GetSettings,
    ApplySettings,
    GetTelemetryId,
    ActivateTelemetry(bool),

    ActivateProcessSuspend(bool),

    RunPrepareShutdown,
    RunPrepareSuspend,
    RunPrepareResume,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum DaemonResponse {
    Settings(CurrentSettings),
    TelemetryId(u64),

}

/// Represents the power profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum PowerProfile {
    Silent = 0x01,
    Default = 0x02,
    Performance = 0x03,
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

/// Represents the keyboard backlight brightness levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum KeyboardBacklightLevel {
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

/// Represents the fan control mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum FanMode {
    Auto,           // Controlled by EC thermal tables
    Full,           // 100% speed override (Turbo)
    Custom(u8),     // Custom PWM duty cycle
}

/// Identifies the specific fan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Hash)]
pub enum FanIndex {
    Cpu = 0x4B,
    Gpu = 0x4D,
}

/// Represents battery charge limit profiles (FlexiCharger)
/// todo: legacy, remove prob
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum ChargeLimit {
    FullCapacity,       // 100%
    HighCapacity,       // 95%
    Balanced,           // 80%
    MaximumLifespan,    // 60%
    DeskMode,           // 40%
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

/// Represents the LED Ring behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum PowerLedMode {
    Auto,
    Custom(u8),
    Animation(BreathConfig),
}


/// Represents breathing animation brightness levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BreathBrightness {
    Max25Percent = 0b00,
    Max50Percent = 0b01,
    Max75Percent = 0b10,
    Max100Percent = 0b11,
}

/// Represents breathing animation step sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BreathStep {
    Slow = 0b00,
    Medium = 0b01,
    Fast = 0b10,
    Instant = 0b11,
}

/// Represents breathing animation delay durations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BreathDelay {
    Ms15 = 0x00,        // ~15.6 ms
    Ms125 = 0x01,       // ~125 ms
    Ms250 = 0x02,       // ~250 ms
    Sec0_5 = 0x03,      // 0.5 sec
    Sec1 = 0x04,        // 1.0 sec
    Sec2 = 0x05,        // 2.0 sec
    Sec4 = 0x06,        // 4.0 sec
}

/// Represents breathing animation configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub struct BreathConfig {
    pub max_brightness: BreathBrightness,
    pub step_up: BreathStep,
    pub step_down: BreathStep,
    pub delay_at_max: BreathDelay,
    pub delay_at_min: BreathDelay,
}

// TODO: move from here! it's should be const vars
impl BreathConfig {
    /// A gentle, smooth breathing effect perfect for normal, idle operation.
    /// Gradually fades in and out with comfortable pauses.
    pub fn smooth() -> Self {
        Self {
            max_brightness: BreathBrightness::Max75Percent,
            step_up: BreathStep::Medium,
            step_down: BreathStep::Medium,
            delay_at_max: BreathDelay::Ms250,
            delay_at_min: BreathDelay::Ms250,
        }
    }

    /// A slow, dim pulse suitable for sleep, standby, or night mode.
    /// Uses low brightness and a long pause at the minimum brightness state.
    pub fn sleep() -> Self {
        Self {
            max_brightness: BreathBrightness::Max25Percent,
            step_up: BreathStep::Slow,
            step_down: BreathStep::Slow,
            delay_at_max: BreathDelay::Sec0_5,
            delay_at_min: BreathDelay::Sec1,
        }
    }

    /// A fast, high-intensity pulse for alerts, errors, or important notifications.
    /// Almost continuous rapid flashing.
    pub fn alert() -> Self {
        Self {
            max_brightness: BreathBrightness::Max75Percent,
            step_up: BreathStep::Instant,
            step_down: BreathStep::Instant,
            delay_at_max: BreathDelay::Ms125,
            delay_at_min: BreathDelay::Ms125,
        }
    }

    /// Deep, relaxing breaths with long holds, similar to meditation routines.
    /// Slowly brightens, holds, slowly dims, and holds again.
    pub fn zen() -> Self {
        Self {
            max_brightness: BreathBrightness::Max50Percent,
            step_up: BreathStep::Slow,
            step_down: BreathStep::Slow,
            delay_at_max: BreathDelay::Sec1,
            delay_at_min: BreathDelay::Sec1,
        }
    }

    /// Sharp, instant burst of light followed by a slow, lingering fade out.
    /// Mimics a heartbeat or a sonar ping.
    pub fn ping() -> Self {
        Self {
            max_brightness: BreathBrightness::Max100Percent,
            step_up: BreathStep::Instant,
            step_down: BreathStep::Slow,
            delay_at_max: BreathDelay::Ms15,
            delay_at_min: BreathDelay::Sec0_5,
        }
    }

    /// A steady, high-energy throb. Good for a device that is actively processing
    /// or compiling something.
    pub fn energetic() -> Self {
        Self {
            max_brightness: BreathBrightness::Max100Percent,
            step_up: BreathStep::Fast,
            step_down: BreathStep::Medium,
            delay_at_max: BreathDelay::Ms125,
            delay_at_min: BreathDelay::Ms125,
        }
    }

    /// A subtle warning state. Bright enough to be noticed, but with
    /// a sharp fade-in and instant fade-out to create a "glitchy" or urgent feel.
    pub fn warning() -> Self {
        Self {
            max_brightness: BreathBrightness::Max100Percent,
            step_up: BreathStep::Fast,
            step_down: BreathStep::Fast,
            delay_at_max: BreathDelay::Ms15,
            delay_at_min: BreathDelay::Ms15,
        }
    }

    /// Slowly builds up tension by gradually increasing brightness to 75%,
    /// holds it for a second, and then instantly snaps to black.
    pub fn vacuum() -> Self {
        Self {
            max_brightness: BreathBrightness::Max75Percent,
            step_up: BreathStep::Slow,
            step_down: BreathStep::Instant, // Sharp drop
            delay_at_max: BreathDelay::Sec1,
            delay_at_min: BreathDelay::Ms250,
        }
    }

    /// Very fast, jagged breathing at high brightness with almost no pauses.
    pub fn panic() -> Self {
        Self {
            max_brightness: BreathBrightness::Max100Percent,
            step_up: BreathStep::Instant,
            step_down: BreathStep::Instant,
            delay_at_max: BreathDelay::Ms15,  // Minimum hardware delay
            delay_at_min: BreathDelay::Ms15,  // Minimum hardware delay
        }
    }

    /// A fast, low-brightness ping in the dark that holds briefly,
    /// then fades away slowly into a long silence.
    pub fn sonar() -> Self {
        Self {
            max_brightness: BreathBrightness::Max25Percent, // Dim
            step_up: BreathStep::Fast,
            step_down: BreathStep::Slow,
            delay_at_max: BreathDelay::Sec0_5,
            delay_at_min: BreathDelay::Sec2,
        }
    }

    /// A medium brightness glow that snaps on fast, but takes a painfully
    /// long time to fade out, creating an unnatural, lingering effect.
    pub fn toxic() -> Self {
        Self {
            max_brightness: BreathBrightness::Max50Percent,
            step_up: BreathStep::Fast,
            step_down: BreathStep::Slow,
            delay_at_max: BreathDelay::Ms250,
            delay_at_min: BreathDelay::Sec1,
        }
    }
}

// todo: prob move from here
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChargeRange { pub min: u8, pub max: u8 }

/// Fixed vocabulary. A new board adds a backend in the daemon, never a variant here.
/// bincode encodes the variant index: append only.
#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargeIntent {
    Full,
    /// None = thresholds are chosen by firmware (N161A).
    Preserve(Option<ChargeRange>),
    Freeze,
}

impl Default for ChargeIntent {
    fn default() -> Self { Self::Full }
}

// ---------- capabilities ----------

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct ChargeCaps {
    pub supported: bool,
    /// Bounds for a user-defined range. None = board cannot do arbitrary ranges.
    pub custom_range: Option<(u8, u8)>,
    /// Firmware-owned (start, stop). Some => Preserve(None) is available.
    pub preserve_fixed: Option<(u8, u8)>,
    /// SoC window where Freeze works. None = unsupported or unverified.
    pub freeze_soc: Option<(u8, u8)>,
    pub survives_daemon: bool,
    /// Already resolved by the daemon for this board, so CLI/GUI never
    /// map a name to an intent themselves and never silently substitute.
    pub presets: Vec<(String, ChargeIntent)>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct FanCaps { pub index: FanIndex, pub duty_max: u8 }

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct KbdCaps { pub on_off: bool, pub levels: bool, pub custom: bool }

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Default)]
pub struct LedCaps { pub on_off: bool, pub brightness: bool, pub animation: bool }

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SensorRole { Cpu, Sys }

/// TODO: pretty heavy. Store in BOX
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct Capabilities {
    pub schema: u16,
    pub board: String,
    pub daemon_version: String,
    pub fans: Vec<FanCaps>,
    pub sensors: Vec<SensorRole>,
    pub power_profiles: Vec<PowerProfile>,
    pub kbd: KbdCaps,
    pub led: LedCaps,
    pub battery_leds: bool,
    pub charge: ChargeCaps,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct ChargeStatus {
    pub desired: ChargeIntent,
    pub effective: ChargeIntent,
    pub soc: u8,
    /// Some(reason) => cannot apply right now; will retry on AC/wake events.
    pub pending: Option<String>,
    pub thresholds: Option<(u8, u8)>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedInfo {
    pub board: String,
    pub chip: Option<String>,
    pub hint: String,
}

// ---------- persisted state ----------

const SETINGS_SCHEMA_VER: u16 = 2;

/// Current configuration settings of the system
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct CurrentSettings {
    pub version: u16,
    pub telemetry_enabled: bool,
    pub telemetry_client_id: u64,
    pub keyboard_backlight: KeyboardBacklightLevel,
    pub led_mode: PowerLedMode,
    pub power_profile: PowerProfile,
    pub charge: ChargeIntent,
    pub fan_mode_cpu: FanMode,
    pub fan_mode_gpu: FanMode,
}

impl Default for CurrentSettings {
    fn default() -> Self {
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let client_id = hasher.finish();

        Self {
            version: SETINGS_SCHEMA_VER,
            telemetry_enabled: true,
            telemetry_client_id: client_id,
            keyboard_backlight: KeyboardBacklightLevel::Medium,
            led_mode: PowerLedMode::Auto,
            power_profile: PowerProfile::Default,
            charge: ChargeIntent::Full,
            fan_mode_cpu: FanMode::Auto,
            fan_mode_gpu: FanMode::Auto,
        }
    }
}

// V1: Old format without motherboard field (for backward compatibility)
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum TelemetryDataV1 {
    Startup { firmware: String, offset: u16, cpu: String, os: String },
    Status { profile: PowerProfile, temps: [u32; 2], fans: [u32; 2] },
    Panic { error: String },
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TelemetryPayloadV1 {
    pub id: u64,
    pub data: TelemetryDataV1,
}

// V2: Current format with motherboard field
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum TelemetryData {
    Startup { firmware: String, offset: u16, cpu: String, os: String, motherboard: String },
    Status { profile: PowerProfile, temps: [u32; 2], fans: [u32; 2] },
    Panic { error: String },
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TelemetryPayload {
    pub id: u64,
    pub data: TelemetryData, // fuck me...
}
