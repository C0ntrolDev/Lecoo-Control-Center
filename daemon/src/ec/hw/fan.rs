use super::EcDevice;
use anyhow::{Result, bail};
use lecoo_types::ec_types::{FanIndex, FanMode};

pub fn apply_fan_mode(ec: &EcDevice, fan: &FanIndex, mode: &FanMode) -> Result<()> {
    let Some(spec) = ec.profile.fan(*fan) else {
        bail!("Board {} has no {:?} fan", ec.profile.id, fan);
    };

    let (policy, duty) = match mode {
        FanMode::Auto => (spec.policy_auto, 0),
        FanMode::Full => (spec.policy_manual, spec.duty_full),
        FanMode::Turbo => (spec.policy_manual, spec.duty_max),
        FanMode::Custom(d) => {
            if *d > spec.duty_max {
                bail!("Requested fan duty cycle ({}) exceeds safe limit ({}).", d, spec.duty_max);
            }
            (spec.policy_manual, *d)
        }
    };

    ec.with_batch(|b| {
        b.write(spec.policy, policy)?;
        b.write(spec.duty, duty)
    })
}
