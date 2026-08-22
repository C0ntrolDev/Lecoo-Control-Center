use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};
use lecoo_types::{caps::*, ec_types::*, settings::CurrentSettings};
use serde::{Deserialize, Serialize};
use std::io::{Read, self, Write};

mod client;
mod server;

pub use client::IpcClient;
pub use server::IpcServer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaemonCommand {
    GetCapabilities,
    RestoreDefaults,
    GetSettings,
    ApplySettings,
    GetTelemetryId,
    ActivateTelemetry(bool),

    ActivateProcessSuspend(bool),

    RunPrepareShutdown, // todo: is this actually used?
    RunPrepareSuspend,
    RunPrepareResume,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum DaemonResponse {
    Settings(Box<CurrentSettings>),
    TelemetryId(u64),
    Capabilities(Box<Capabilities>),
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum IpcRequest {
    /// Request the current telemetry and configuration state
    GetSystemState,
    GetFansRPM,
    GetTemperatures,
    GetChargeStatus,
    /// TODO: LEGACY api for reading charge limit
    GetChargeLimit,
    GetPowerProfile,
    GetKeyboardBacklight,

    /// Apply a new power profile (Silent/Default/Performance)
    SetPowerProfile(PowerProfile),

    /// Set a specific fan's mode (Auto/Full/Custom)
    SetFanMode {
        fan: FanIndex,
        mode: FanMode,
    },

    /// Set keyboard backlight brightness
    SetKeyboardBacklight(KeyboardBacklightLevel),

    /// Set battery charge threshold
    SetChargeIntent(ChargeIntent),
    /// TODO: LEGACY api for changing charge
    SetChargeLimit(ChargeLimit),

    /// Control the LED Ring
    SetLedMode(PowerLedMode),

    /// Send a command to the daemon
    DaemonCommand(DaemonCommand),

    #[serde(other)]
    Unknown,
}


/// Responses sent FROM the Daemon TO the Client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum IpcResponse {
    Ok,
    Error(IpcError),

    /// Information about the embedded controller
    SystemInfo(SystemInfo),

    /// RPM readings for both fans
    FanRpm { cpu: u16, gpu: u16 },

    /// Temperature readings for CPU and System
    Temps { cpu_c: u8, sys_c: u8 },

    /// Current battery charge limit (TODO: LEGACY)
    ChargeLimit { min: u8, max: u8, current: u8 },

    /// Current battery charge status
    ChargeStatus(ChargeStatus),

    /// Current keyboard backlight brightness
    KeyboardBacklight(KeyboardBacklightLevel),

    /// Current power profile
    PowerLimit(PowerProfile),

    /// Response from the daemon
    DaemonResponse(DaemonResponse),

    /// Information about telemetry being disabled
    TelemetryDisabledInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    Internal,
    /// Daemon does not support this request or the client is too new.
    UnsupportedRequest,
    /// Daemon does not support the hardware the client is using.
    UnsupportedHardware,
    /// Feature exists but preconditions are unmet. Text goes into a GUI tooltip.
    Precondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemInfo {
    pub chip: String,
    pub revision: String,
    pub hram_offset: u16,
    pub daemon_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsupported: Option<UnsupportedInfo>,
}

impl IpcError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), unsupported: None }
    }
}

// ---------

pub struct IpcConnection {
    stream: Stream,
}

impl IpcConnection {
    pub fn accept_handshake(&mut self) -> io::Result<()> {
        let mut req = [0u8; 5];
        self.stream.read_exact(&mut req)?;

        if &req[0..3] != b"LCC" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic bytes"));
        }

        let client_major_ver = req[3];
        let client_minor_ver = req[4];

        if client_major_ver != IPC_PROTOCOL_VERSION[0] || client_minor_ver != IPC_PROTOCOL_VERSION[1] {
            let resp = [b'E', b'R', b'R', IPC_PROTOCOL_VERSION[0], IPC_PROTOCOL_VERSION[1]];
            let _ = self.stream.write_all(&resp);
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Version mismatch. Client: v{}.{}, Server: v{}.{}",
                    client_major_ver, client_minor_ver,
                    IPC_PROTOCOL_VERSION[0], IPC_PROTOCOL_VERSION[1]
                )
            ));
        }

        let resp = [b'O', b'K', b'K', IPC_PROTOCOL_VERSION[0], IPC_PROTOCOL_VERSION[1]];
        self.stream.write_all(&resp)?;

        Ok(())
    }

    pub fn send<T: Encode>(&mut self, msg: &T) -> io::Result<()> {
        let data = bincode::encode_to_vec(msg, config::standard())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let len = data.len() as u32;

        self.stream.write_all(&len.to_le_bytes())?;
        self.stream.write_all(&IPC_PROTOCOL_VERSION)?;
        self.stream.write_all(&data)?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn recv<T: Decode<()>>(&mut self) -> io::Result<T> {
        let mut len_bytes = [0u8; 4];
        let mut bytes_read = 0;

        while bytes_read < 4 {
            let n = self.stream.read(&mut len_bytes[bytes_read..])?;
            if n == 0 {
                if bytes_read == 0 {
                    return Err(io::Error::new(io::ErrorKind::ConnectionReset, "Connection reset by peer"));
                } else {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection dropped while reading"));
                }
            }
            bytes_read += n;
        }

        let len = u32::from_le_bytes(len_bytes) as usize;

        let mut msg_version = [0u8; 3];
        self.stream.read_exact(&mut msg_version)?;

        if msg_version[0] != IPC_PROTOCOL_VERSION[0] || msg_version[1] != IPC_PROTOCOL_VERSION[1] {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "IPC protocol version mismatch"));
        }

        if len > 5 * 1024 * 1024 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "IPC payload too large"));
        }

        let mut data = vec![0u8; len];
        self.stream.read_exact(&mut data)?;

        let (msg, _) = bincode::decode_from_slice(&data, config::standard())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(msg)
    }
}

fn get_socket_name() -> io::Result<interprocess::local_socket::Name<'static>> {
    "lecoo_ctl_daemon"
        .to_ns_name::<GenericNamespaced>()
        .map(|n| n.into_owned())
}

const fn parse_u8(s: &str) -> u8 {
    let mut res: u8 = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        res = res * 10 + (bytes[i] - b'0');
        i += 1;
    }
    res
}
