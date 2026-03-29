//! AG-UI protocol support.
//!
//! a2native can act as an AG-UI **frontend tool handler**: it receives a stream
//! of AG-UI events (`TOOL_CALL_START` → `TOOL_CALL_ARGS` → `TOOL_CALL_END`),
//! assembles the tool call arguments (which must be a valid a2native or Google
//! A2UI form spec), renders the form natively, and emits a `TOOL_CALL_RESULT`
//! event that the agent can consume.
//!
//! Spec: <https://github.com/ag-ui-protocol/ag-ui>

use serde::{Deserialize, Serialize};

// ── Input types ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AGUIEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(rename = "toolCallId")]
    tool_call_id: Option<String>,
    #[serde(rename = "toolCallName")]
    tool_call_name: Option<String>,
    delta: Option<String>,
    #[serde(rename = "threadId")]
    thread_id: Option<String>,
    #[serde(rename = "runId")]
    run_id: Option<String>,
}

/// Contextual information extracted from an AG-UI event stream.
pub struct AGUIContext {
    pub tool_call_id: String,
    pub tool_call_name: String,
    pub thread_id: Option<String>,
    pub run_id: Option<String>,
}

// ── Output types ───────────────────────────────────────────────────────────

/// An AG-UI `TOOL_CALL_RESULT` event — the response from a frontend tool.
#[derive(Serialize)]
pub struct AGUIOutput {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "messageId")]
    pub message_id: String,
    #[serde(rename = "toolCallId")]
    pub tool_call_id: String,
    pub content: String,
    pub role: String,
}

// ── Detection & parsing ────────────────────────────────────────────────────

/// Returns `true` if the input looks like an AG-UI event stream.
/// Detection heuristic: contains the literal string `"TOOL_CALL_START"`.
pub fn is_agui(input: &str) -> bool {
    input.contains("\"TOOL_CALL_START\"")
}

/// Parse an AG-UI JSONL event stream.
///
/// Scans for `RUN_STARTED`, `TOOL_CALL_START`, and `TOOL_CALL_ARGS` events,
/// concatenates all arg deltas into a single JSON string, and returns it
/// together with contextual information.
///
/// The caller is responsible for further parsing the returned args string
/// (which may be a2native legacy format or Google A2UI format).
pub fn parse(json: &str) -> Result<(String, AGUIContext), String> {
    let mut tool_call_id = String::new();
    let mut tool_call_name = String::new();
    let mut thread_id: Option<String> = None;
    let mut run_id: Option<String> = None;
    let mut args = String::new();

    for result in serde_json::Deserializer::from_str(json).into_iter::<AGUIEvent>() {
        let event = result.map_err(|e| format!("AG-UI parse error: {e}"))?;
        match event.event_type.as_str() {
            "RUN_STARTED" => {
                thread_id = event.thread_id;
                run_id = event.run_id;
            }
            "TOOL_CALL_START" => {
                if let Some(id) = event.tool_call_id {
                    tool_call_id = id;
                }
                if let Some(name) = event.tool_call_name {
                    tool_call_name = name;
                }
            }
            "TOOL_CALL_ARGS" => {
                if let Some(delta) = event.delta {
                    args.push_str(&delta);
                }
            }
            _ => {}
        }
    }

    if tool_call_id.is_empty() {
        return Err("No TOOL_CALL_START found in AG-UI event stream".into());
    }

    Ok((args, AGUIContext { tool_call_id, tool_call_name, thread_id, run_id }))
}

// ── Output conversion ──────────────────────────────────────────────────────

/// Build a `TOOL_CALL_RESULT` event from a JSON result string and the context.
///
/// `content_json` is the already-serialized result payload (e.g., a
/// `userAction` object or legacy `{status, values}` object, as a JSON string).
pub fn to_output(content_json: &str, ctx: &AGUIContext) -> AGUIOutput {
    AGUIOutput {
        event_type: "TOOL_CALL_RESULT".into(),
        message_id: format!("{}-result", ctx.tool_call_id),
        tool_call_id: ctx.tool_call_id.clone(),
        content: content_json.to_string(),
        role: "tool".into(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_agui_positive() {
        let input = r#"{"type":"TOOL_CALL_START","toolCallId":"tc1","toolCallName":"show_form"}"#;
        assert!(is_agui(input));
    }

    #[test]
    fn test_is_agui_negative_legacy() {
        let input = r#"{"title":"Deploy","components":[]}"#;
        assert!(!is_agui(input));
    }

    #[test]
    fn test_is_agui_negative_a2ui() {
        let input = r#"{"surfaceUpdate":{"surfaceId":"s1","components":[]}}"#;
        assert!(!is_agui(input));
    }

    #[test]
    fn test_parse_single_line_jsonl() {
        let input = concat!(
            r#"{"type":"RUN_STARTED","threadId":"t1","runId":"r1"}"#, "\n",
            r#"{"type":"TOOL_CALL_START","toolCallId":"tc1","toolCallName":"show_form"}"#, "\n",
            r#"{"type":"TOOL_CALL_ARGS","toolCallId":"tc1","delta":"{\"title\":\"Deploy\"}"}"#, "\n",
            r#"{"type":"TOOL_CALL_END","toolCallId":"tc1"}"#
        );
        let (args, ctx) = parse(input).expect("parse should succeed");
        assert_eq!(args, r#"{"title":"Deploy"}"#);
        assert_eq!(ctx.tool_call_id, "tc1");
        assert_eq!(ctx.tool_call_name, "show_form");
        assert_eq!(ctx.thread_id.as_deref(), Some("t1"));
        assert_eq!(ctx.run_id.as_deref(), Some("r1"));
    }

    #[test]
    fn test_parse_streaming_args_concatenated() {
        let input = concat!(
            r#"{"type":"TOOL_CALL_START","toolCallId":"tc2","toolCallName":"a2n_form"}"#, "\n",
            r#"{"type":"TOOL_CALL_ARGS","toolCallId":"tc2","delta":"{\"title\":"}"#, "\n",
            r#"{"type":"TOOL_CALL_ARGS","toolCallId":"tc2","delta":"\"Confirm\"}"}"#, "\n",
            r#"{"type":"TOOL_CALL_END","toolCallId":"tc2"}"#
        );
        let (args, ctx) = parse(input).expect("parse should succeed");
        assert_eq!(args, r#"{"title":"Confirm"}"#);
        assert_eq!(ctx.tool_call_id, "tc2");
    }

    #[test]
    fn test_parse_no_tool_call_error() {
        let input = r#"{"type":"RUN_STARTED","threadId":"t1","runId":"r1"}"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn test_to_output() {
        let ctx = AGUIContext {
            tool_call_id: "tc1".into(),
            tool_call_name: "show_form".into(),
            thread_id: Some("t1".into()),
            run_id: Some("r1".into()),
        };
        let result = r#"{"status":"submitted","values":{"env":"prod"}}"#;
        let out = to_output(result, &ctx);
        assert_eq!(out.event_type, "TOOL_CALL_RESULT");
        assert_eq!(out.tool_call_id, "tc1");
        assert_eq!(out.message_id, "tc1-result");
        assert_eq!(out.role, "tool");
        assert_eq!(out.content, result);

        let json = serde_json::to_string(&out).unwrap();
        assert!(json.contains("\"TOOL_CALL_RESULT\""));
        assert!(json.contains("\"toolCallId\":\"tc1\""));
        assert!(json.contains("\"messageId\":\"tc1-result\""));
    }

    #[test]
    fn test_ignores_other_events() {
        let input = concat!(
            r#"{"type":"TEXT_MESSAGE_START","messageId":"m1","role":"assistant"}"#, "\n",
            r#"{"type":"TEXT_MESSAGE_CONTENT","messageId":"m1","delta":"Fill this form:"}"#, "\n",
            r#"{"type":"TEXT_MESSAGE_END","messageId":"m1"}"#, "\n",
            r#"{"type":"TOOL_CALL_START","toolCallId":"tc3","toolCallName":"form"}"#, "\n",
            r#"{"type":"TOOL_CALL_ARGS","toolCallId":"tc3","delta":"{\"title\":\"Test\"}"}"#, "\n",
            r#"{"type":"TOOL_CALL_END","toolCallId":"tc3"}"#
        );
        let (args, ctx) = parse(input).expect("parse should succeed");
        assert_eq!(args, r#"{"title":"Test"}"#);
        assert_eq!(ctx.tool_call_id, "tc3");
    }
}
