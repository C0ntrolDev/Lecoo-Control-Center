use ipc::Capabilities;
use rust_i18n::t;

use std::io::IsTerminal;

#[inline]
pub fn bold() -> &'static str { if std::io::stdout().is_terminal() { "\x1b[1m" } else { "" } }
#[inline]
pub fn bold_underline() -> &'static str { if std::io::stdout().is_terminal() { "\x1b[1;4m" } else { "" } }

#[inline]
pub fn err_red() -> &'static str { if std::io::stderr().is_terminal() { "\x1b[31m" } else { "" } }
#[inline]
pub fn err_bold() -> &'static str { if std::io::stderr().is_terminal() { "\x1b[1m" } else { "" } }
#[inline]
pub fn err_bold_red() -> &'static str { if std::io::stderr().is_terminal() { "\x1b[1;31m" } else { "" } }
#[inline]
pub fn err_reset() -> &'static str { if std::io::stderr().is_terminal() { "\x1b[0m" } else { "" } }
#[inline]
pub fn reset() -> &'static str { if std::io::stdout().is_terminal() { "\x1b[0m" } else { "" } }

pub fn bin_name() -> String {
    std::env::args()
        .next()
        .and_then(|p| std::path::Path::new(&p).file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "lecoo-ctrl".into())
}

pub fn get_usage(target: Option<&str>, is_err: bool) -> String {
    let bold_under = if is_err { err_bold_red() } else { bold_underline() };
    let res = if is_err { err_reset() } else { reset() };
    let usage_prefix = format!("{}{}{} ", bold_under, t!("help_usage"), res);
    let bin = bin_name();
    match target.map(|s| s.to_lowercase()).as_deref() {
        Some("info") => format!("{usage_prefix}{bin} info"),
        Some("temps") => format!("{usage_prefix}{bin} temps"),
        Some("fans") => format!("{usage_prefix}{bin} fans"),
        Some("monitoring") => format!("{usage_prefix}{bin} monitoring [--rate <SECONDS>]"),
        Some("fan") => format!("{usage_prefix}{bin} fan <TARGET> <MODE> [PWM_VAL]"),
        Some("charge") => format!("{usage_prefix}{bin} charge [<PRESET> | --min <N> --max <N>]"),
        Some("power") => format!("{usage_prefix}{bin} power [PROFILE]"),
        Some("kbd") => format!("{usage_prefix}{bin} kbd [MODE] [PWM_VAL]"),
        Some("led") => format!("{usage_prefix}{bin} led <auto|custom> [PWM_VAL]"),
        Some("daemon") => format!("{usage_prefix}{bin} daemon <SUBCOMMAND>"),
        _ => format!("{usage_prefix}{bin} <COMMAND> [OPTIONS]"),
    }
}

pub fn print_usage(target: Option<&str>) {
    println!("{}", get_usage(target, false));
}

pub fn print_help(target: Option<&str>, caps: Option<&Capabilities>) {
    match target.map(|s| s.to_lowercase()).as_deref() {
        Some("info") => {
            println!("{}", t!("cmd_info_about"));
            println!();
            print_usage(target);
            println!();
            print_options();
        }
        Some("temps") => {
            println!("{}", t!("cmd_temps_about"));
            println!();
            print_usage(target);
            println!();
            print_options();
        }
        Some("fans") => {
            println!("{}", t!("cmd_fans_about"));
            println!();
            print_usage(target);
            println!();
            print_options();
        }
        Some("monitoring") => {
            println!("{}", t!("cmd_monitoring_about"));
            println!();
            print_usage(target);
            println!();
            println!("{}{} {}", bold_underline(), t!("help_options"), reset());
            println!("  {}--rate{} <SECONDS>  {}", bold(), reset(), t!("arg_monitoring_rate_help"));
            println!("  {}-h, --help{}        {}", bold(), reset(), t!("help_cmd_help_about"));
        }
        Some("fan") => {
            println!("{}", t!("cmd_fan_about"));
            println!();
            print_usage(target);
            println!();
            println!("{}{} {}", bold_underline(), t!("help_arguments"), reset());
            println!("  {}<TARGET>{}", bold(), reset());
            println!("          {}", t!("arg_fan_target_help"));
            println!();
            println!("          {}", t!("help_possible_values"));
            println!("          - {}cpu{}:  {}", bold(), reset(), t!("arg_fan_target_cpu_help"));
            println!("          - {}gpu{}:  {}", bold(), reset(), t!("arg_fan_target_gpu_help"));
            println!("          - {}both{}: {}", bold(), reset(), t!("arg_fan_target_both_help"));
            println!();
            println!("  {}<MODE>{}", bold(), reset());
            println!("          {}", t!("arg_fan_mode_help"));
            println!();
            println!("          {}", t!("help_possible_values"));
            println!("          - {}auto{}:   {}", bold(), reset(), t!("arg_fan_mode_auto_help"));
            println!("          - {}full{}:   {}", bold(), reset(), t!("arg_fan_mode_full_help"));
            println!("          - {}turbo{}:  {}", bold(), reset(), t!("arg_fan_mode_turbo_help"));
            println!("          - {}custom{}: {}", bold(), reset(), t!("arg_fan_mode_custom_help"));
            println!();
            println!("  {}[PWM_VAL]{}", bold(), reset());
            println!("          {}", t!("arg_fan_val_help"));
            println!();
            print_options();
        }
        Some("charge") => {
            println!("{}", t!("cmd_charge_about"));
            println!();
            print_usage(target);
            println!();
            println!("{}{} {}", bold_underline(), t!("help_arguments"), reset());
            println!("  {}<PRESET>{}", bold(), reset());
            println!("          {}", t!("arg_charge_limit_long_help").lines().next().unwrap_or("Preset name"));
            println!();
            println!("          {}", t!("help_possible_values"));

            if let Some(c) = caps {
                if !c.charge.presets.is_empty() {
                    for (name, _) in &c.charge.presets {
                        let desc = match name.as_str() {
                            "full" => t!("arg_charge_limit_full_help"),
                            "high" => t!("arg_charge_limit_high_help"),
                            "balanced" => t!("arg_charge_limit_balanced_help"),
                            "lifespan" => t!("arg_charge_limit_lifespan_help"),
                            "desk" => t!("arg_charge_limit_desk_help"),
                            "freeze" | "hold" => t!("arg_charge_limit_freeze_help"),
                            _ => name.as_str().into(),
                        };
                        println!("          - {}{:<10}{} {}", bold(), name, reset(), desc);
                    }
                }
                if let Some((min, max)) = c.charge.custom_range {
                    println!();
                    println!("  {}--min <N> --max <N>{}", bold(), reset());
                    println!("          {} {min}% .. {max}%", t!("help_custom_range"));
                }
            } else {
                println!("          - {}full{}:     {}", bold(), reset(), t!("arg_charge_limit_full_help"));
                println!("          - {}high{}:     {}", bold(), reset(), t!("arg_charge_limit_high_help"));
                println!("          - {}balanced{}: {}", bold(), reset(), t!("arg_charge_limit_balanced_help"));
                println!("          - {}lifespan{}: {}", bold(), reset(), t!("arg_charge_limit_lifespan_help"));
                println!("          - {}desk{}:     {}", bold(), reset(), t!("arg_charge_limit_desk_help"));
                println!("          - {}freeze{}:   {}", bold(), reset(), t!("arg_charge_limit_freeze_help"));
            }
            println!();
            print_options();
        }
        Some("power") => {
            println!("{}", t!("cmd_power_about"));
            println!();
            print_usage(target);
            println!();
            println!("{}{} {}", bold_underline(), t!("help_arguments"), reset());
            println!("  {}[PROFILE]{}", bold(), reset());
            println!("          {}", t!("arg_power_profile_help"));
            println!();
            println!("          {}", t!("help_possible_values"));
            println!("          - {}silent{}:  {}", bold(), reset(), t!("arg_power_profile_silent_help"));
            println!("          - {}default{}: {}", bold(), reset(), t!("arg_power_profile_default_help"));
            println!("          - {}perf{}:    {}", bold(), reset(), t!("arg_power_profile_perf_help"));
            println!();
            print_options();
        }
        Some("kbd") => {
            println!("{}", t!("cmd_kbd_about"));
            println!();
            print_usage(target);
            println!();
            println!("{}{} {}", bold_underline(), t!("help_arguments"), reset());
            println!("  {}[MODE]{}", bold(), reset());
            println!("          {}", t!("arg_kbd_mode_help"));
            println!();
            println!("          {}", t!("help_possible_values"));
            println!("          - {}off{}:    {}", bold(), reset(), t!("arg_kbd_mode_off_help"));
            println!("          - {}low{}:    {}", bold(), reset(), t!("arg_kbd_mode_low_help"));
            println!("          - {}medium{}: {}", bold(), reset(), t!("arg_kbd_mode_medium_help"));
            println!("          - {}high{}:   {}", bold(), reset(), t!("arg_kbd_mode_high_help"));
            println!("          - {}custom{}: {}", bold(), reset(), t!("arg_kbd_mode_custom_help"));
            println!();
            println!("  {}[PWM_VAL]{}", bold(), reset());
            println!("          {}", t!("arg_kbd_val_help"));
            println!();
            print_options();
        }
        Some("led") => {
            println!("{}", t!("cmd_led_about"));
            println!();
            print_usage(target);
            println!();
            println!("{}{} {}", bold_underline(), t!("help_arguments"), reset());
            println!("  {}<auto|custom>{}", bold(), reset());
            println!("          {}", t!("help_possible_values"));
            println!("          - {}auto{}:   {}", bold(), reset(), t!("cmd_led_auto_about"));
            println!("          - {}custom{}: {}", bold(), reset(), t!("cmd_led_custom_about"));
            println!();
            println!("  {}[PWM_VAL]{}", bold(), reset());
            println!("          {}", t!("arg_led_val_help"));
            println!();
            print_options();
        }
        Some("daemon") => {
            println!("{}", t!("cmd_daemon_about"));
            println!();
            print_usage(target);
            println!();
            println!("{}{} {}", bold_underline(), t!("help_subcommands"), reset());
            println!("  {}telemetry <enable|disable|id>{}  {}", bold(), reset(), t!("cmd_daemon_telemetry_about"));
            println!("  {}settings <reset|read|apply>{}    {}", bold(), reset(), t!("cmd_daemon_settings_about"));
            println!("  {}version{}                        {}", bold(), reset(), t!("cmd_daemon_version_about"));
            println!();
            print_options();
        }
        _ => print_global_help(caps),
    }
}

fn print_options() {
    println!("{}{} {}", bold_underline(), t!("help_options"), reset());
    println!("  {}-h, --help{}  {}", bold(), reset(), t!("help_cmd_help_about"));
}

fn print_global_help(caps: Option<&Capabilities>) {
    println!("{}{}{}", bold(), t!("cmd_app_about"), reset());
    println!();
    print_usage(None);
    println!();
    println!("{}{} {}", bold_underline(), t!("help_commands"), reset());
    println!("  {}info{}          {}", bold(), reset(), t!("cmd_info_about"));
    println!("  {}temps{}         {}", bold(), reset(), t!("cmd_temps_about"));
    println!("  {}fans{}          {}", bold(), reset(), t!("cmd_fans_about"));
    println!("  {}monitoring{}    {}", bold(), reset(), t!("cmd_monitoring_about"));
    println!("  {}fan{}           {}", bold(), reset(), t!("cmd_fan_about"));
    println!("  {}charge{}        {}", bold(), reset(), t!("cmd_charge_about"));
    println!("  {}power{}         {}", bold(), reset(), t!("cmd_power_about"));
    println!("  {}kbd{}           {}", bold(), reset(), t!("cmd_kbd_about"));
    println!("  {}led{}           {}", bold(), reset(), t!("cmd_led_about"));
    println!("  {}daemon{}        {}", bold(), reset(), t!("cmd_daemon_about"));
    println!("  {}help{}          {}", bold(), reset(), t!("help_cmd_help_about"));
    println!();
    print_options();
    println!();

    if let Some(c) = caps {
        println!("{}{}Hardware Capabilities (Board: {}) {}", bold_underline(), bold(), c.board, reset());

        if c.charge.supported {
            if !c.charge.presets.is_empty() {
                let names: Vec<&str> = c.charge.presets.iter().map(|(n, _)| n.as_str()).collect();
                println!("  {}{}{} {}", bold(), t!("help_charge_presets"), reset(), names.join(", "));
            }
        }

        if !c.power_profiles.is_empty() {
            let profiles: Vec<String> = c.power_profiles.iter().map(|p| format!("{p:?}")).collect();
            println!("  {}{}{} {}", bold(), t!("help_power_profiles"), reset(), profiles.join(", "));
        }
    } else {
        println!("{}", t!("help_note_offline"));
    }
}
