//! Forwards `log::error!` records into telemetry.

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use lecoo_types::telemetry::TelemetryData;
use log::{Level, Log, Metadata, Record};

/// One report per call site per process, capped. An EC read failing inside a
/// loop would otherwise turn a single bug into a telemetry flood.
const MAX_DISTINCT: usize = 20;
static SEEN: Mutex<Vec<u64>> = Mutex::new(Vec::new());

/// Set while the panic hook runs. The hook logs the panic *and* emits a
/// dedicated `Panic` event; without this the same text would arrive twice.
pub(super) static IN_PANIC: AtomicBool = AtomicBool::new(false);

struct Forwarder {
    inner: Box<dyn Log>,
}

impl Log for Forwarder {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn flush(&self) {
        self.inner.flush();
    }

    fn log(&self, record: &Record) {
        self.inner.log(record);

        if record.level() != Level::Warn || IN_PANIC.load(Ordering::Relaxed) {
            return;
        }

        let module = record.module_path().unwrap_or("<unknown>");
        if is_own_module(module) {
            return;
        }

        let line = record.line().unwrap_or(0);
        if !first_time(module, line) {
            return;
        }

        super::send(TelemetryData::ErrorLog {
            message: record.args().to_string(),
            module: module.to_owned(),
            line,
        });
    }
}

fn is_own_module(module: &str) -> bool {
    match module_path!().rsplit_once("::") {
        Some((telemetry_root, _)) => module.starts_with(telemetry_root),
        None => false,
    }
}

fn first_time(module: &str, line: u32) -> bool {
    let mut hasher = DefaultHasher::new();
    module.hash(&mut hasher);
    line.hash(&mut hasher);
    let key = hasher.finish();

    let Ok(mut seen) = SEEN.lock() else { return false };
    if seen.len() >= MAX_DISTINCT || seen.contains(&key) {
        return false;
    }
    seen.push(key);

    true
}

/// Installs `inner` as the global logger, wrapped so error records also reach telemetry
pub fn install(inner: Box<dyn Log>, level: log::LevelFilter) {
    if log::set_boxed_logger(Box::new(Forwarder { inner })).is_err() {
        eprintln!("Logger already installed, telemetry error forwarding is off");
        return;
    }
    log::set_max_level(level);
}
