use std::{panic, sync::atomic::Ordering};

use lecoo_types::telemetry::TelemetryData;

use super::errors::IN_PANIC;

pub fn install() {
    panic::set_hook(Box::new(|info| {
        let (file, line) = info
            .location()
            .map(|l| (l.file().to_owned(), l.line()))
            .unwrap_or_else(|| ("<unknown>".to_owned(), 0));

        let message = info
            .payload()
            .downcast_ref::<&'static str>()
            .map(|s| (*s).to_owned())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown panic message".to_owned());

        // Thread name is what tells a panicked EC worker apart from a panicked
        // IPC connection handler, and the backtrace never survives to the server.
        let thread = std::thread::current().name().map(str::to_owned);

        IN_PANIC.store(true, Ordering::Relaxed);
        log::error!("CRITICAL PANIC in file '{file}' at line {line}: {message}");
        log::logger().flush();
        IN_PANIC.store(false, Ordering::Relaxed);

        super::send(TelemetryData::Panic { message, file, line, thread });
    }));
}
