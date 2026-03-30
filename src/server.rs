//! HTTP/SSE and WebSocket servers for external agent connectivity.
//!
//! Agents can send form specs via HTTP POST to `/form` (SSE) or as WebSocket
//! text messages, and receive the user's response back on the same connection.
//!
//! Both servers share the same IPC channel as the CLI session protocol, so
//! they participate in the same serialized queue of forms shown in the window.
//!
//! SSE endpoint
//! ------------
//!   POST http://127.0.0.1:<PORT>/form   — body: JSON form spec (any supported format)
//!   Response: text/event-stream
//!     event: waiting          — form is now displayed
//!     data: {"status":"waiting"}
//!
//!     event: result           — user submitted / cancelled / timed out
//!     data: <a2native output JSON>
//!
//!   GET  http://127.0.0.1:<PORT>/health — liveness probe → {"status":"ok"}
//!
//! WebSocket endpoint
//! ------------------
//!   ws://127.0.0.1:<PORT>
//!   Client → text frame:  JSON form spec (any supported format)
//!   Server → text frame:  {"type":"waiting","status":"waiting"}
//!   Server → text frame:  {"type":"result","data":{...a2native output...}}
//!   Connection stays open; client can send multiple form specs in sequence.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::Sender;

/// Maximum accepted request body size (1 MB). Prevents memory exhaustion via a
/// malicious Content-Length header.
const MAX_BODY_BYTES: usize = 1024 * 1024;

use crate::protocol::A2NOutput;
use crate::renderer::egui_impl::IpcCommand;

// ── HTTP / SSE server ─────────────────────────────────────────────────────────

/// Bind an HTTP server on `port` (0 = OS-assigned). Returns the actual port.
pub fn start_sse(port: u16, tx: Sender<IpcCommand>) -> u16 {
    let listener =
        TcpListener::bind(("127.0.0.1", port)).expect("Failed to bind SSE port");
    let actual_port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(s) = stream {
                let tx2 = tx.clone();
                std::thread::spawn(move || handle_sse_conn(s, tx2));
            }
        }
    });
    actual_port
}

fn handle_sse_conn(stream: std::net::TcpStream, tx: Sender<IpcCommand>) {
    let cloned = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut reader = BufReader::new(cloned);

    // Parse HTTP request line
    let mut req_line = String::new();
    if reader.read_line(&mut req_line).is_err() {
        return;
    }
    let parts: Vec<&str> = req_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0].to_ascii_uppercase();
    let path = parts[1].to_string();

    // Read headers; collect Content-Length
    let mut content_length: usize = 0;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).is_err() {
            break;
        }
        let ht = h.trim().to_ascii_lowercase();
        if ht.is_empty() {
            break;
        }
        if let Some(v) = ht.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }

    let mut stream = stream;

    match (method.as_str(), path.as_str()) {
        ("POST", "/form") => {
            // Reject oversized bodies before allocating (prevents memory exhaustion).
            if content_length > MAX_BODY_BYTES {
                let _ = stream.write_all(
                    b"HTTP/1.1 413 Content Too Large\r\n\
                      Content-Type: text/plain\r\n\r\n\
                      Request body exceeds 1 MB limit",
                );
                return;
            }
            // Read body
            let mut body = vec![0u8; content_length];
            if std::io::Read::read_exact(&mut reader, &mut body).is_err() {
                return;
            }
            let body_str = String::from_utf8_lossy(&body).into_owned();

            let input = match parse_input_json(&body_str) {
                Ok(i) => i,
                Err(e) => {
                    let msg = format!(
                        "HTTP/1.1 400 Bad Request\r\n\
                         Content-Type: text/plain\r\n\r\n{e}"
                    );
                    let _ = stream.write_all(msg.as_bytes());
                    return;
                }
            };

            let (resp_tx, resp_rx) = std::sync::mpsc::sync_channel::<A2NOutput>(0);
            if tx.send(IpcCommand::Update { input, response_tx: resp_tx }).is_err() {
                let _ = stream.write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\n\r\n",
                );
                return;
            }

            // SSE response headers
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\n\
                  Content-Type: text/event-stream\r\n\
                  Cache-Control: no-cache\r\n\
                  Connection: keep-alive\r\n\r\n",
            );

            // Acknowledge: form is now displayed
            let _ = stream.write_all(b"event: waiting\ndata: {\"status\":\"waiting\"}\n\n");

            // Block until user submits, then emit result
            if let Ok(output) = resp_rx.recv() {
                let json = serde_json::to_string(&output).unwrap_or_default();
                let event = format!("event: result\ndata: {json}\n\n");
                let _ = stream.write_all(event.as_bytes());
            }
        }
        ("GET", "/health") => {
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\n\
                  Content-Type: application/json\r\n\r\n\
                  {\"status\":\"ok\"}",
            );
        }
        _ => {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n");
        }
    }
}

// ── WebSocket server ──────────────────────────────────────────────────────────

/// Bind a WebSocket server on `port` (0 = OS-assigned). Returns the actual port.
pub fn start_ws(port: u16, tx: Sender<IpcCommand>) -> u16 {
    let listener =
        TcpListener::bind(("127.0.0.1", port)).expect("Failed to bind WS port");
    let actual_port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(s) = stream {
                let tx2 = tx.clone();
                std::thread::spawn(move || handle_ws_conn(s, tx2));
            }
        }
    });
    actual_port
}

fn handle_ws_conn(stream: std::net::TcpStream, tx: Sender<IpcCommand>) {
    use tungstenite::{accept, Message};

    let mut ws: tungstenite::WebSocket<std::net::TcpStream> = match accept(stream) {
        Ok(ws) => ws,
        Err(_) => return,
    };

    // A single WS connection can handle multiple sequential forms.
    loop {
        match ws.read() {
            Ok(Message::Text(txt)) => {
                let input = match parse_input_json(&txt) {
                    Ok(i) => i,
                    Err(e) => {
                        let _ = ws.send(Message::Text(
                            format!("{{\"error\":\"{e}\"}}"),
                        ));
                        continue;
                    }
                };

                let (resp_tx, resp_rx) = std::sync::mpsc::sync_channel::<A2NOutput>(0);
                if tx.send(IpcCommand::Update { input, response_tx: resp_tx }).is_err() {
                    let _ = ws.send(Message::Text(
                        r#"{"error":"daemon closed"}"#.to_string(),
                    ));
                    break;
                }

                let _ = ws.send(Message::Text(
                    r#"{"type":"waiting","status":"waiting"}"#.to_string(),
                ));

                if let Ok(output) = resp_rx.recv() {
                    let json = serde_json::to_string(&output).unwrap_or_default();
                    let msg = format!(r#"{{"type":"result","data":{json}}}"#);
                    let _ = ws.send(Message::Text(msg));
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
}

// ── Shared input parsing ──────────────────────────────────────────────────────

/// Parse a form spec string using the same auto-detection as the CLI:
/// AG-UI envelope → Google A2UI JSONL → a2native flat JSON.
fn parse_input_json(json: &str) -> Result<crate::protocol::A2NInput, String> {
    let form_json = if crate::protocol::agui::is_agui(json) {
        match crate::protocol::agui::parse(json) {
            Ok((args, _ctx)) => args,
            Err(e) => return Err(format!("AG-UI parse error: {e}")),
        }
    } else {
        json.to_string()
    };

    if crate::protocol::a2ui::is_a2ui(&form_json) {
        crate::protocol::a2ui::parse(&form_json)
            .map(|(input, _ctx)| input)
            .map_err(|e| format!("A2UI parse error: {e}"))
    } else {
        serde_json::from_str::<crate::protocol::A2NInput>(&form_json)
            .map_err(|e| format!("JSON parse error: {e}"))
    }
}
