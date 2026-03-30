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

/// Sanitise the caller-supplied UUID so it is safe to embed in a filename.
///
/// Keeps only ASCII alphanumerics, hyphens, and underscores (all of which are
/// path-safe on every supported OS).  This prevents path traversal while
/// preserving a deterministic, readable mapping — unlike a hash, the output
/// won't silently diverge across Rust versions or builds.
fn safe_uuid_component(uuid: &str) -> String {
    let s: String = uuid
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    if s.is_empty() {
        // Derive a stable hex digest so different invalid inputs don't collide
        // on the same port file (e.g. "" and "!!!" must not both become "default").
        let hash = uuid.bytes().fold(0u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64));
        format!("inv-{hash:016x}")
    } else {
        s
    }
}

fn port_file(uuid: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("a2n-session-{}.port", safe_uuid_component(uuid)));
    p
}

pub fn write_port(uuid: &str, port: u16) {
    use std::fs::OpenOptions;

    let path = port_file(uuid);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true); // fail if file already exists (no overwrite race)

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600); // owner-only read/write
    }

    if let Ok(mut file) = options.open(&path) {
        let _ = write!(file, "{port}");
    }
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
    // Avoid panicking on serialization failure — return None instead.
    let msg = serde_json::to_string(&IpcMessage::Update { input: input.clone() }).ok()?;
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
            // Best-effort: skip if serialization somehow fails.
            if let Ok(msg) = serde_json::to_string(&IpcMessage::Close) {
                let _ = writeln!(stream, "{msg}");
            }
        }
    }
    remove_port(uuid);
}

/// Poll until the daemon writes its port file *and* accepts a TCP connection,
/// or the timeout elapses.
///
/// The caller is responsible for removing any stale port file **before** spawning
/// the daemon, so this function does not need to do it.
pub fn wait_for_daemon(uuid: &str, timeout_secs: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    while Instant::now() < deadline {
        if let Some(port) = read_port(uuid) {
            // Validate readiness with an actual TCP connection attempt.
            if TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}
