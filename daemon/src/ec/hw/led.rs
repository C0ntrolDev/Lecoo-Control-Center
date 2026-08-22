use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Result, bail};
use ipc::PowerLedMode;
use super::EcDevice;
use crate::ec::{Addr, LedOps, PwmSpec};

static IS_LED_ALREADY_CUSTOM: AtomicBool = AtomicBool::new(false);

pub fn apply_led_mode(ec: &EcDevice, mode: &PowerLedMode) -> Result<()> {
    match ec.profile.led {
        LedOps::None => bail!("Board {} has no controllable power LED", ec.profile.id),

        LedOps::Pwm { pwm } => match mode {
            PowerLedMode::Auto => set_auto(ec, pwm, None),
            PowerLedMode::Custom(brightness) => set_custom(ec, pwm, None, *brightness),
            PowerLedMode::Animation(_) =>
                bail!("Board {} has no hardware LED animation", ec.profile.id),
        },

        LedOps::PwmBreath { pwm, breath_en, breath_step, breath_delay } => match mode {
            PowerLedMode::Auto => set_auto(ec, pwm, Some(breath_en)),
            PowerLedMode::Custom(brightness) => set_custom(ec, pwm, Some(breath_en), *brightness),
            PowerLedMode::Animation(config) => {
                enter_custom(ec, pwm)?;
                reset_engine(ec, pwm, Some(breath_en))?;

                ec.with_batch(|b| {
                    b.write(pwm.prescaler, 0x00)?;
                    b.write(pwm.cycle, 0xFF)?;
                    b.write(breath_step, breath_step_register(&config))?;
                    b.write(breath_delay, breath_delay_register(&config))?;
                    b.write(breath_en, 0x01)
                })
            }
        },
    }
}

pub fn apply_battery_leds(ec: &EcDevice, orange_on: bool, white_on: bool) -> Result<()> {
    let Some(spec) = ec.profile.battery_leds else { return Ok(()) };

    let mut port = ec.read(spec.port)?;
    let set = |on: bool, mask: u8, port: &mut u8| {
        if on == spec.active_low { *port &= !mask } else { *port |= mask }
    };
    set(orange_on, spec.orange_mask, &mut port);
    set(white_on, spec.white_mask, &mut port);

    ec.write(spec.port, port)
}

// ------ helpers ------

/// Creates the value for the breath_step register (LCR1)
#[inline]
pub fn breath_step_register(c: &BreathConfig) -> u8 {
    ((c.max_brightness as u8) << 4)
        | ((c.step_down as u8) << 2)
        | (c.step_up as u8)
}

/// Creates the value for the breath_delay register (LCR2)
#[inline]
pub fn breath_delay_register(c: &BreathConfig) -> u8 {
    ((c.delay_at_max as u8) << 4) | (c.delay_at_min as u8)
}


#[inline]
fn enter_custom(ec: &EcDevice, pwm: PwmSpec) -> Result<()> {
    if !IS_LED_ALREADY_CUSTOM.load(Ordering::Relaxed) {
        ec.with_batch(|b| {
            b.write(pwm.bypass, 0x01)?;
            b.write(pwm.mux, 0x00)
        })?;
        IS_LED_ALREADY_CUSTOM.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[inline]
fn reset_engine(ec: &EcDevice, pwm: PwmSpec, breath_en: Option<Addr>) -> Result<()> {
    ec.with_batch(|b| {
        if let Some(en) = breath_en {
            b.write(en, 0x00)?;
        }
        b.write(pwm.prescaler, 0x00)?;
        b.write(pwm.cycle, 0xFF)
    })
}

#[inline]
pub fn reset_led_anim_engine(ec: &EcDevice) -> Result<()> {
    match ec.profile.led {
        LedOps::None => Ok(()),
        LedOps::Pwm { pwm } => reset_engine(ec, pwm, None),
        LedOps::PwmBreath { pwm, breath_en, .. } => reset_engine(ec, pwm, Some(breath_en)),
    }
}

#[inline]
fn set_auto(ec: &EcDevice, pwm: PwmSpec, breath_en: Option<Addr>) -> Result<()> {
    ec.write(pwm.bypass, 0x00)?;
    reset_engine(ec, pwm, breath_en)?;
    IS_LED_ALREADY_CUSTOM.store(false, Ordering::Relaxed);
    Ok(())
}

#[inline]
fn set_custom(ec: &EcDevice, pwm: PwmSpec, breath_en: Option<Addr>, brightness: u8) -> Result<()> {
    enter_custom(ec, pwm)?;
    reset_engine(ec, pwm, breath_en)?;
    ec.write(pwm.duty, brightness)
}
