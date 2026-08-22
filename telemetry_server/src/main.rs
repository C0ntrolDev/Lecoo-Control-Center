use log::{error, info, warn};
use rusqlite::Connection;
use simplelog::SimpleLogger;
use std::{
    io::Read,
    sync::{Arc, Mutex},
    thread,
};
use tiny_http::{Method, Request, Response, Server};

use telemetry_server::{db, ingest, legacy};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
static SERVER_ADDR: &str = "127.0.0.1:8368";
const MAX_BODY_SIZE: usize = 512 * 1024;

/// Which decoder a request gets
enum Wire {
    V2,
    Legacy,
}

fn main() {
    SimpleLogger::init(log::LevelFilter::Info, simplelog::Config::default())
        .expect("Failed to initialize logger");

    let db = Arc::new(Mutex::new(db::open("telemetry.db")));

    let server = Server::http(SERVER_ADDR).expect("Failed to start server");
    info!("Telemetry server ({}) listening on http://{}", VERSION, SERVER_ADDR);

    for request in server.incoming_requests() {
        let db = Arc::clone(&db);
        thread::spawn(move || handle(request, &db));
    }
}

fn handle(mut request: Request, db: &Mutex<Connection>) {
    let method = request.method().clone();
    let path = request.url().split('?').next().unwrap_or("").to_owned();

    let status = match (method, path.as_str()) {
        (Method::Get, "/telemetry/health") => 200,
        (Method::Post, "/telemetry") => ingest(&mut request, db, Wire::Legacy),
        (Method::Post, "/telemetry/v2") => ingest(&mut request, db, Wire::V2),
        _ => 404,
    };

    let _ = request.respond(Response::empty(status));
}

fn ingest(request: &mut Request, db: &Mutex<Connection>, wire: Wire) -> u16 {
    let version = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("X-Daemon-Version"))
        .map(|h| h.value.as_str().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let declared = request.body_length().unwrap_or(0);
    if declared > MAX_BODY_SIZE {
        return 413;
    }

    // `body_length` is None for a chunked body, and that reader is an unbounded
    // stream, so the cap has to be enforced on the read itself. One byte past
    // the limit is enough to tell "at the limit" from "over it".
    let mut body = Vec::with_capacity(declared.min(MAX_BODY_SIZE));
    if request.as_reader().take(MAX_BODY_SIZE as u64 + 1).read_to_end(&mut body).is_err() {
        warn!("Failed to read request body");
        return 400;
    }
    if body.len() > MAX_BODY_SIZE {
        return 413;
    }

    // A panic in one request thread poisons the lock; the connection behind it
    // is still usable, so recovering keeps that from turning into a permanent
    // 500 for every later request.
    let conn = db.lock().unwrap_or_else(|poisoned| {
        error!("Recovering DB lock poisoned by an earlier panic");
        poisoned.into_inner()
    });

    match wire {
        Wire::V2 => ingest::store(&conn, &version, &body),
        Wire::Legacy => legacy::store(&conn, &version, &body),
    }
}
