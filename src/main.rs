mod protocol;
mod renderer;
mod server;
mod session;

use std::io::{self, IsTerminal, Read};

use clap::{Parser, Subcommand};

const SCHEMA: &str = include_str!("../schema/a2native-v0.1.schema.json");

/// a2n — native desktop UI renderer for AI agents.
///
/// Reads a JSON form spec (from stdin or as an inline argument), renders a
/// native window, and writes the user's response as JSON to stdout.
/// One JSON in, one JSON out — no chat loop, no web server.
///
/// Run `a2n schema` to print the full input JSON Schema.
#[derive(Parser)]
#[command(
    name = "a2n",
    version,
    about = "Native desktop UI renderer for AI agents — collect user input via native OS forms",
    long_about = None,
    disable_help_subcommand = true,
)]
struct Cli {
    /// Subcommand: `help` or `schema`.
    #[command(subcommand)]
    command: Option<Command>,

    /// Inline JSON form spec (alternative to piping via stdin).
    #[arg(value_name = "JSON", conflicts_with = "command")]
    json: Option<String>,

    /// Session UUID — keeps the window alive for multi-turn interactions.
    /// On the first call a daemon is spawned; subsequent calls reuse it.
    #[arg(long, value_name = "UUID", conflicts_with = "close")]
    session: Option<String>,

    /// Close a running session window and exit.
    #[arg(long, value_name = "UUID", conflicts_with = "session")]
    close: Option<String>,

    /// [Internal] Run as session daemon — do not call directly.
    #[arg(long, hide = true, requires = "session", conflicts_with = "close")]
    daemon: bool,

    /// Expose an HTTP/SSE endpoint on this port for external agents (requires --session).
    /// POST /form → streams text/event-stream with 'waiting' then 'result' events.
    #[arg(long, value_name = "PORT", conflicts_with = "close")]
    sse: Option<u16>,

    /// Expose a WebSocket endpoint on this port for external agents (requires --session).
    /// Send form spec as text frame, receive {"type":"result","data":{...}} when submitted.
    #[arg(long, value_name = "PORT", conflicts_with = "close")]
    ws: Option<u16>,
}

#[derive(Subcommand)]
enum Command {
    /// Show this help message.
    Help,
    /// Print the a2native input JSON Schema and exit.
    Schema,
}

fn print_help() {
    println!(
        "\
a2n  —  native desktop UI renderer for AI agents

WHAT IT DOES
  AI agents often need structured input from a user (choices, file paths,
  configurations).  Back-and-forth chat is slow and error-prone.  a2n solves
  this: the agent generates a JSON form spec, a2n renders a native window,
  the user fills in the fields, and the answers come back as JSON.

  Input priority: inline JSON arg  →  stdin pipe  →  (neither → this help)

USAGE
  a2n [JSON]                     One-shot: inline JSON form spec
  echo '{{...}}' | a2n           One-shot: JSON form spec via stdin pipe
  a2n [JSON] --session <UUID>    Session: keep window open across turns
  a2n --close <UUID>             Close a session window
  a2n schema                     Print the input JSON Schema
  a2n --help                     Show full flag reference
  a2n --version                  Show version

  (Running a2n with no JSON and no stdin pipe shows this help.)

SESSION MODE
  Use --session <UUID> to keep a window alive across multiple agent turns.
  The first invocation spawns a background daemon; subsequent calls update
  the form in the existing window.  Close with: a2n --close <UUID>

HTTP/SSE & WEBSOCKET  (external agent API)
  Add --sse <PORT> and/or --ws <PORT> to a session to expose endpoints
  that external agents (Python, Node, etc.) can connect to directly —
  no CLI wrapper needed.

  a2n --session <UUID> --sse <PORT>   Daemon with HTTP/SSE on PORT
  a2n --session <UUID> --ws  <PORT>   Daemon with WebSocket on PORT

  SSE  POST http://127.0.0.1:<PORT>/form    (body = JSON form spec)
       GET  http://127.0.0.1:<PORT>/health  (liveness probe)
       Response: text/event-stream
         event: waiting   data: {{status:waiting}}
         event: result    data: {{...a2native output JSON...}}

  WS   ws://127.0.0.1:<PORT>
       Send:    JSON form spec text frame (a2native / A2UI / AG-UI)
       Receive: {{type:waiting}}  then  {{type:result, data:{{...}}}}
       One connection supports multiple sequential forms.

  Both endpoints auto-detect input format (a2native, Google A2UI, AG-UI).
  Close the daemon with: a2n --close <UUID>

SCHEMA
  Run `a2n schema` to see the full JSON Schema for the a2native input format.
  Online: https://a2native.github.io/schema/a2native-v0.1.schema.json

  a2n auto-detects the input format (priority order):
    1. AG-UI  — TOOL_CALL_START/ARGS/END event stream  → TOOL_CALL_RESULT output
    2. A2UI   — Google A2UI v0.8 surfaceUpdate JSONL   → userAction output
    3. Legacy — a2native flat JSON format              → {{status, values}} output

  AG-UI spec:     https://github.com/ag-ui-protocol/ag-ui
  Google A2UI:    https://github.com/google/a2ui

SECURITY
  Every window displays a permanent banner warning the user that the
  interface was generated by an AI agent and that their input will be sent
  to the agent.  Never enter passwords or private keys into a2n forms.

EXAMPLES
  # Simple confirmation dialog
  a2n '{{\"title\":\"Deploy?\",\"components\":[
    {{\"id\":\"ok\",\"type\":\"button\",\"label\":\"Deploy\",\"action\":\"submit\"}},
    {{\"id\":\"no\",\"type\":\"button\",\"label\":\"Cancel\",\"action\":\"cancel\"}}
  ]}}'

  # Multi-field form via stdin
  cat form.json | a2n

  # Multi-turn wizard (CLI session)
  echo '{{...step1...}}' | a2n --session my-wizard-abc123
  echo '{{...step2...}}' | a2n --session my-wizard-abc123
  a2n --close my-wizard-abc123

  # Start SSE daemon for an external Python/Node agent
  a2n --session my-session --sse 8080
  # Agent POSTs to http://127.0.0.1:8080/form and reads SSE result

  # Start WebSocket daemon
  a2n --session my-session --ws 8081
  # Agent connects to ws://127.0.0.1:8081 and sends form specs as text frames
"
    );
}

fn main() {
    let cli = Cli::parse();

    // ── Subcommands ───────────────────────────────────────────────────────────
    match &cli.command {
        Some(Command::Help) => {
            print_help();
            return;
        }
        Some(Command::Schema) => {
            println!("{SCHEMA}");
            return;
        }
        None => {}
    }

    // ── Close a running session ───────────────────────────────────────────────
    if let Some(uuid) = &cli.close {
        session::send_close(uuid);
        return;
    }

    // ── Daemon mode (spawned internally) ─────────────────────────────────────
    if cli.daemon {
        let uuid = cli.session.as_deref().unwrap_or_else(|| {
            eprintln!("error: --daemon requires --session <UUID>");
            std::process::exit(1);
        });
        renderer::egui_impl::run_daemon(uuid, cli.sse, cli.ws);
        return;
    }

    // ── Resolve JSON input: inline arg takes priority, then stdin ─────────────
    let input_json: Option<String> = if let Some(json) = cli.json {
        Some(json)
    } else if !io::stdin().is_terminal() {
        // stdin is a pipe / redirected file — read it
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .expect("Failed to read stdin");
        Some(buf)
    } else {
        None
    };

    // No input: valid only when --session + (--sse | --ws) = server-only mode.
    if input_json.is_none() && cli.close.is_none() {
        let server_mode = (cli.sse.is_some() || cli.ws.is_some()) && cli.session.is_some();
        if !server_mode {
            print_help();
            return;
        }
    }

    // ── Detect format and parse input (only when input is present) ────────────
    //
    // Priority: AG-UI envelope → Google A2UI JSONL → a2native legacy JSON

    enum InputFormat {
        Agui(protocol::agui::AGUIContext),
        Other, // A2UI or legacy — a2ui_ctx distinguishes them
    }

    struct ParsedInput {
        input: protocol::A2NInput,
        fmt: InputFormat,
        a2ui_ctx: Option<protocol::a2ui::A2UIContext>,
    }

    let parsed: Option<ParsedInput> = match input_json {
        Some(raw) => {
            let (form_json, fmt): (String, InputFormat) =
                if protocol::agui::is_agui(&raw) {
                    match protocol::agui::parse(&raw) {
                        Ok((args, ctx)) => (args, InputFormat::Agui(ctx)),
                        Err(e) => {
                            eprintln!("error: could not parse AG-UI input: {e}");
                            std::process::exit(1);
                        }
                    }
                } else {
                    (raw, InputFormat::Other)
                };

            let (input, a2ui_ctx) = if protocol::a2ui::is_a2ui(&form_json) {
                match protocol::a2ui::parse(&form_json) {
                    Ok((a2n_input, ctx)) => (a2n_input, Some(ctx)),
                    Err(e) => {
                        eprintln!("error: could not parse A2UI form spec: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                let a2n_input: protocol::A2NInput =
                    serde_json::from_str(&form_json).unwrap_or_else(|e| {
                        eprintln!("error: could not parse input JSON: {e}");
                        std::process::exit(1);
                    });
                (a2n_input, None)
            };

            Some(ParsedInput { input, fmt, a2ui_ctx })
        }
        None => None,
    };

    /// Emit output in the correct envelope format.
    fn emit_output(
        output: protocol::A2NOutput,
        fmt: &InputFormat,
        a2ui_ctx: Option<&protocol::a2ui::A2UIContext>,
    ) {
        let inner_json = if let Some(ctx) = a2ui_ctx {
            let a2ui_out = protocol::a2ui::to_output(&output, ctx);
            serde_json::to_string(&a2ui_out).unwrap()
        } else {
            serde_json::to_string(&output).unwrap()
        };

        match fmt {
            InputFormat::Agui(ctx) => {
                let agui_out = protocol::agui::to_output(&inner_json, ctx);
                println!("{}", serde_json::to_string(&agui_out).unwrap());
            }
            InputFormat::Other => {
                println!("{inner_json}");
            }
        }
    }

    // ── Session mode ─────────────────────────────────────────────────────────
    if let Some(uuid) = &cli.session {
        let sse_port = cli.sse;
        let ws_port = cli.ws;

        if let Some(p) = parsed {
            // Submit a form to the daemon (spawning it if needed).
            if let Some(output) = session::try_send(uuid, &p.input) {
                emit_output(output, &p.fmt, p.a2ui_ctx.as_ref());
                return;
            }

            session::remove_port(uuid);
            spawn_daemon(uuid, sse_port, ws_port);
            if !session::wait_for_daemon(uuid, 10) {
                eprintln!("error: timed out waiting for session daemon to start");
                std::process::exit(1);
            }

            let output = session::try_send(uuid, &p.input).unwrap_or_else(|| {
                eprintln!("error: failed to connect to session daemon");
                std::process::exit(1);
            });
            emit_output(output, &p.fmt, p.a2ui_ctx.as_ref());
        } else {
            // Server-only mode: spawn daemon and exit.  External agents connect
            // via SSE / WebSocket; this process just starts the daemon.
            session::remove_port(uuid);
            spawn_daemon(uuid, sse_port, ws_port);
            if let Some(port) = sse_port {
                eprintln!("a2n: SSE  http://127.0.0.1:{port}/form  (session: {uuid})");
            }
            if let Some(port) = ws_port {
                eprintln!("a2n: WS   ws://127.0.0.1:{port}/  (session: {uuid})");
            }
            eprintln!("a2n: close with  a2n --close {uuid}");
        }
        return;
    }

    // ── One-shot (stateless) mode ─────────────────────────────────────────────
    let p = parsed.expect("input_json is Some in one-shot mode");
    let renderer = renderer::egui_impl::EguiRenderer::new();
    let output = renderer::Renderer::run(renderer, p.input);
    emit_output(output, &p.fmt, p.a2ui_ctx.as_ref());
}

fn spawn_daemon(uuid: &str, sse_port: Option<u16>, ws_port: Option<u16>) {
    let exe = std::env::current_exe().expect("Cannot determine executable path");
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--daemon").arg("--session").arg(uuid);
    if let Some(port) = sse_port {
        cmd.arg("--sse").arg(port.to_string());
    }
    if let Some(port) = ws_port {
        cmd.arg("--ws").arg(port.to_string());
    }

    // On Windows, detach from the parent console so no extra window pops up.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    cmd.spawn().expect("Failed to spawn session daemon");
}

