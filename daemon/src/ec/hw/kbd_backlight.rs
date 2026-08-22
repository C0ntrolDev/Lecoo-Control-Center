use anyhow::{Result, bail};
use lecoo_types::ec_types::KeyboardBacklightLevel;
use super::EcDevice;
use crate::ec::KbdOps;

pub fn read_keyboard_backlight(ec: &EcDevice) -> Result<KeyboardBacklightLevel> {
    use KeyboardBacklightLevel as L;
    match ec.profile.kbd {
        KbdOps::None => bail!("Board {} has no controllable keyboard backlight", ec.profile.id),

        KbdOps::PwmDuty { reg } => Ok(match ec.read(reg)? {
            0x00 => L::Off,
            0x4C => L::Low,
            0x99 => L::Medium,
            0xFF => L::High,
            v => L::Custom(v),
        }),

        // todo: not every revision supports all levels!!!!! fix it later
        KbdOps::Levels { reg, custom_val, .. } => match ec.read(reg)? {
            0x00 => Ok(L::Off),
            0x01 => Ok(L::Low),
            0x02 => Ok(L::Medium),
            0x03 => Ok(L::High),
            0xFF => Ok(L::Custom(ec.read(custom_val)?)),
            v => bail!("Invalid keyboard backlight level: {:#04x}", v),
        },
    }
}

pub fn apply_keyboard_backlight(ec: &EcDevice, level: &KeyboardBacklightLevel) -> Result<()> {
    use KeyboardBacklightLevel as L;
    match ec.profile.kbd {
        KbdOps::None => bail!("Board {} has no controllable keyboard backlight", ec.profile.id),

        KbdOps::PwmDuty { reg } => {
            let value = match level {
                L::Off => 0x00,
                L::Low => 0x4C,
                L::Medium => 0x99,
                L::High => 0xFF,
                L::Custom(v) => *v,
            };
            ec.write(reg, value)
        }

        KbdOps::Levels { reg, custom_val, bypass_timeout, mux } => match level {
            L::Off => ec.write(reg, 0x00),
            L::Low => ec.write(reg, 0x01),
            L::Medium => ec.write(reg, 0x02),
            L::High => ec.write(reg, 0x03),
            L::Custom(v) => ec.with_batch(|b| {
                b.write(bypass_timeout, 0xFF)?;
                b.write(mux, 0x00)?;
                b.write(reg, 0xFF)?;
                b.write(custom_val, *v)
            }),
        },
    }
}
