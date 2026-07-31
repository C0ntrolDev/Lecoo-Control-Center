use anyhow::{Context, Result, bail};
use ipc::{IpcResponse, PowerProfile};
use super::EcDevice;

pub fn apply_power_profile(ec: &EcDevice, profile: &PowerProfile) -> Result<IpcResponse> {
    let Some(spec) = ec.profile.power else {
        bail!("Board {} has no power profiles", ec.profile.id);
    };

    let raw = spec.map.iter()
        .find(|(_, p)| p == profile)
        .map(|(v, _)| *v)
        .with_context(|| format!("Profile {:?} is unavailable on {}", profile, ec.profile.id))?;

    ec.write(spec.reg, raw)?;
    Ok(IpcResponse::Success)
}

pub fn read_power_profile(ec: &EcDevice) -> Result<PowerProfile> {
    let Some(spec) = ec.profile.power else {
        bail!("Board {} has no power profiles", ec.profile.id);
    };

    let raw = ec.read(spec.reg)?;
    spec.map.iter()
        .find(|(v, _)| *v == raw)
        .map(|(_, p)| *p)
        .with_context(|| format!("Unknown power profile: {}", raw))
}
