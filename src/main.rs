mod protocol;
mod renderer;
mod session;

use std::io::{self, Read};

use clap::Parser;

/// a2native (a2n) — A2UI Protocol implementation.
///
/// Reads a JSON form spec from stdin, renders a native window, and emits the
/// user's response as JSON to stdout.  One JSON in, one JSON out — no chat loop.
#[derive(Parser)]
#[command(
    name = "a2n",
    version,
    about = "A2UI Protocol — collect user input via native UI forms for AI agents",
    long_about = "a2native implements the A2UI protocol:\n  \
                  read a JSON form spec from stdin → show a native window → write the\n  \
                  user's answers as JSON to stdout.\n\n  \
                  Use --session <UUID> for long-lifecycle windows that stay open across\n  \
                  multiple agent turns.  The UUID is caller-supplied; any unique string works."
)]
struct Cli {
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
}

fn main() {
    let cli = Cli::parse();

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
        renderer::egui_impl::run_daemon(uuid);
        return;
    }

    // ── Normal / session client mode ─────────────────────────────────────────
    let mut input_json = String::new();
    io::stdin()
        .read_to_string(&mut input_json)
        .expect("Failed to read stdin");

    let input: protocol::A2NInput = serde_json::from_str(&input_json).unwrap_or_else(|e| {
        eprintln!("error: could not parse input JSON: {e}");
        std::process::exit(1);
    });

    if let Some(uuid) = &cli.session {
        // Try to forward to an already-running session daemon.
        if let Some(output) = session::try_send(uuid, &input) {
            println!("{}", serde_json::to_string(&output).unwrap());
            return;
        }

        // No daemon running yet — clear any stale port file, then spawn.
        session::remove_port(uuid);
        spawn_daemon(uuid);
        if !session::wait_for_daemon(uuid, 10) {
            eprintln!("error: timed out waiting for session daemon to start");
            std::process::exit(1);
        }

        let output = session::try_send(uuid, &input).unwrap_or_else(|| {
            eprintln!("error: failed to connect to session daemon");
            std::process::exit(1);
        });
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        // Stateless one-shot mode.
        let renderer = renderer::egui_impl::EguiRenderer::new();
        let output = renderer::Renderer::run(renderer, input);
        println!("{}", serde_json::to_string(&output).unwrap());
    }
}

fn spawn_daemon(uuid: &str) {
    let exe = std::env::current_exe().expect("Cannot determine executable path");
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--daemon").arg("--session").arg(uuid);

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

