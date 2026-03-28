//! Session management for long-lifecycle windows.
//!
//! Each session is identified by a caller-supplied UUID string.  A tiny file in
//! the system temp directory records the TCP port of the running daemon, making
//! the daemon discoverable by subsequent CLI invocations with the same UUID.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::protocol::{A2NInput, A2NOutput, IpcMessage};

// ── Port file helpers ─────────────────────────────────────────────────────────

fn port_file(uuid: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("a2n-session-{uuid}.port"));
    p
}

pub fn write_port(uuid: &str, port: u16) {
    let _ = std::fs::write(port_file(uuid), port.to_string());
}

pub fn read_port(uuid: &str) -> Option<u16> {
    std::fs::read_to_string(port_file(uuid))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

pub fn remove_port(uuid: &str) {
    let _ = std::fs::remove_file(port_file(uuid));
}

// ── Client-side IPC ───────────────────────────────────────────────────────────

/// Connect to an existing session daemon and send a new form.
/// Returns `None` if no session exists or the connection fails.
pub fn try_send(uuid: &str, input: &A2NInput) -> Option<A2NOutput> {
    let port = read_port(uuid)?;
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).ok()?;
    // 5-minute read timeout – the user may take a while to fill the form.
    let _ = stream.set_read_timeout(Some(Duration::from_secs(300)));
    let msg = serde_json::to_string(&IpcMessage::Update { input: input.clone() }).unwrap();
    writeln!(stream, "{msg}").ok()?;
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str(line.trim()).ok()
}

/// Ask a running session daemon to close its window.
pub fn send_close(uuid: &str) {
    if let Some(port) = read_port(uuid) {
        if let Ok(mut stream) = TcpStream::connect(format!("127.0.0.1:{port}")) {
            let msg = serde_json::to_string(&IpcMessage::Close).unwrap();
            let _ = writeln!(stream, "{msg}");
        }
    }
    remove_port(uuid);
}

/// Poll until the daemon has written its port file, or the timeout elapses.
pub fn wait_for_daemon(uuid: &str, timeout_secs: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    while Instant::now() < deadline {
        if read_port(uuid).is_some() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}
