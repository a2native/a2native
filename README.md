# a2native

**A2UI Protocol** reference implementation — collect user input via native UI forms for AI agents.

One JSON in → native window → one JSON out. No chat loop. No web server.

> ⚠ **Security notice** — every window rendered by a2native displays a warning banner.
> See [Security Considerations](#security-considerations) before deploying.

---

## The Problem

AI agents often need to gather structured input from a human — a choice, a file path, a date range, a
multi-step configuration wizard. The typical solution is a back-and-forth chat loop: the agent asks a
question, the user types an answer, repeat. This is slow, error-prone, and unpleasant for anything more
than a single field.

**a2native solves this with native UI forms:**

```
Agent generates a JSON form spec
          ↓
    a2n reads it from stdin
          ↓
  Native window appears (egui)
          ↓
 User fills in fields & clicks Submit
          ↓
   JSON result written to stdout
          ↓
     Agent reads the answers
```

A complex wizard that would take 10 rounds of chat can become a single form.

---

## A2UI Protocol

a2native implements the **A2UI protocol** — a JSON-based contract between AI agents and native UI
renderers.

### Input schema

```jsonc
{
  "title":   "My Form",          // optional window title
  "timeout": 60,                 // optional auto-close after N seconds
  "theme": {                     // optional
    "dark_mode": true,
    "accent_color": "#6C63FF"
  },
  "components": [ /* ... */ ]    // required
}
```

### Output schema

```jsonc
{
  "status": "submitted" | "cancelled" | "timeout",
  "values": {
    "field_id": "user value",    // string, number, bool, or array
    // ...
  }
}
```

### Component reference

#### Display

| type | required fields | description |
|---|---|---|
| `text` | `id`, `content` | Plain text label |
| `markdown` | `id`, `content` | Headings (`#`/`##`/`###`) and `**bold**` |
| `image` | `id`, `src` | Image from file path or URL; `alt` optional |
| `divider` | `id` | Horizontal separator |

#### Input

| type | key fields | output value type |
|---|---|---|
| `text-field` | `label`, `placeholder`, `required`, `default_value` | `string` |
| `textarea` | `label`, `placeholder`, `required`, `default_value` | `string` |
| `number-input` | `label`, `min`, `max`, `step`, `default_value` | `number` |
| `date-picker` | `label`, `required`, `default_value` (YYYY-MM-DD) | `string` |
| `time-picker` | `label`, `required`, `default_value` (HH:MM) | `string` |
| `dropdown` | `label`, `options`, `required`, `default_value` | `string` |
| `checkbox` | `label`, `default_value` | `boolean` |
| `checkbox-group` | `label`, `options`, `default_values` | `string[]` |
| `radio-group` | `label`, `options`, `required`, `default_value` | `string` |
| `slider` | `label`, `min` (0), `max` (100), `step`, `default_value` | `number` |
| `file-upload` | `label`, `accept`, `multiple` | `string` (path, `;`-separated if multiple) |

`options` / `default_values` use `{ "value": "...", "label": "..." }` objects.

#### Action

| type | key fields | description |
|---|---|---|
| `button` | `label`, `action` | `action`: `"submit"` (default), `"cancel"`, `"custom"` |

#### Layout

| type | key fields | description |
|---|---|---|
| `card` | `title`, `children` | Bordered group; children are any components |

---

## Installation

### Pre-built binaries

Download the latest release from [GitHub Releases](https://github.com/a2native/a2native/releases).

### Build from source

```bash
git clone https://github.com/a2native/a2native.git
cd a2native
cargo build --release
# Binary: target/release/a2n (or a2n.exe on Windows)
```

Requires Rust ≥ 1.75.

---

## Usage

### One-shot mode

```bash
echo '{"title":"Deploy","components":[
  {"id":"env","type":"dropdown","label":"Environment",
   "options":[{"value":"prod","label":"Production"},
              {"value":"stag","label":"Staging"}]},
  {"id":"confirm","type":"checkbox","label":"I have reviewed the changes"},
  {"id":"go","type":"button","label":"Deploy","action":"submit"}
]}' | a2n
```

Output (after user interaction):

```json
{"status":"submitted","values":{"env":"prod","confirm":true}}
```

### Session mode — long-lifecycle windows

Use `--session <UUID>` to keep a window open across multiple agent turns.
The window stays alive between submissions, updating its form each time the
agent calls `a2n --session <UUID>`.

```bash
# Turn 1 — first form
echo '{"title":"Step 1 of 3","components":[...]}' | a2n --session my-wizard-abc123
# → {"status":"submitted","values":{...}}

# Turn 2 — same window, new form
echo '{"title":"Step 2 of 3","components":[...]}' | a2n --session my-wizard-abc123
# → {"status":"submitted","values":{...}}

# Turn 3
echo '{"title":"Step 3 of 3","components":[...]}' | a2n --session my-wizard-abc123

# Done — close the window
a2n --close my-wizard-abc123
```

**How it works:**

The first `--session` invocation spawns a background daemon that owns the
window.  Subsequent invocations are short-lived clients that connect to the
daemon via a local TCP socket, send the new form JSON, and wait for the result.
This is the same client-daemon pattern used by [agent-browser](https://github.com/vercel-labs/agent-browser).

Session port files are stored in the system temp directory:
`{TMPDIR}/a2n-session-<UUID>.port`

### Close a session

```bash
a2n --close <UUID>
```

The window closes and the daemon exits cleanly.

### Help

```bash
a2n --help
a2n --version
```

---

## Full example

```jsonc
{
  "title": "New Project Wizard",
  "timeout": 120,
  "theme": { "dark_mode": true, "accent_color": "#6C63FF" },
  "components": [
    { "id": "h",    "type": "markdown", "content": "## Set up your project" },
    { "id": "name", "type": "text-field", "label": "Project name",
      "placeholder": "my-app", "required": true },
    { "id": "lang", "type": "radio-group", "label": "Language",
      "options": [
        {"value":"rust","label":"Rust"},
        {"value":"ts","label":"TypeScript"},
        {"value":"py","label":"Python"}
      ], "required": true },
    { "id": "oss",  "type": "checkbox", "label": "Open source", "default_value": true },
    { "id": "div",  "type": "divider" },
    { "id": "ok",   "type": "button", "label": "Create Project", "action": "submit" },
    { "id": "no",   "type": "button", "label": "Cancel", "action": "cancel" }
  ]
}
```

---

## Security Considerations

a2native is designed to be called by AI agents automatically.  This creates
real risks that users and integrators must understand:

### Warning banner

**Every window rendered by a2native displays a permanent amber banner:**

> ⚠ This interface was generated by an AI agent — do not enter sensitive
> information unless you trust the source.

This banner cannot be suppressed by the form spec.

### Known risks

| Risk | Description |
|---|---|
| **Prompt injection via form content** | A compromised agent could generate a form that visually mimics a trusted application (e.g., fake OS password dialog, fake bank login). |
| **Credential harvesting** | A malicious agent could ask users to type passwords, API keys, or personal data, then exfiltrate them. |
| **Social engineering** | Markdown and text components allow arbitrary messaging; a bad actor could craft persuasive text to trick users. |
| **Session hijacking** | If a session UUID is guessable or reused insecurely, another process on the same machine could send forms to an open session. |

### Mitigations

- **Only run a2native from agents you trust** — treat it like executing arbitrary code.
- Use short-lived `timeout` values to minimise exposure of idle session windows.
- Never enter passwords, private keys, or financial data into a2native forms.
- Prefer random, unguessable UUIDs for sessions (e.g., UUIDs v4).
- On multi-user systems, be aware that session port files in `TMPDIR` may be
  visible to other users.

---

## License

Apache-2.0 — see [LICENSE](LICENSE).

| | |
|---|---|
| Protocol | A2UI |
| Renderer | [egui](https://github.com/emilk/egui) 0.29 |
| File picker | [rfd](https://github.com/PolyMeilex/rfd) 0.15 |
| CLI | [clap](https://github.com/clap-rs/clap) 4 |
