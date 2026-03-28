//! Google A2UI protocol support — input parsing and output formatting.
//!
//! Accepts A2UI v0.8+ `surfaceUpdate` / `beginRendering` JSONL as input and
//! emits the A2UI `userAction` client-to-server format as output.
//!
//! Reference: <https://github.com/google/a2ui>

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::{A2NInput, A2NOutput, ButtonAction, Component, OutputStatus, SelectOption, Theme};

// ── Server-to-client message types ────────────────────────────────────────────

/// A single A2UI server-to-client JSONL message.
/// A valid message contains exactly one of the action properties.
#[derive(Debug, Deserialize)]
pub struct A2UIMessage {
    #[serde(rename = "surfaceUpdate")]
    pub surface_update: Option<SurfaceUpdate>,
    #[serde(rename = "beginRendering")]
    pub begin_rendering: Option<BeginRendering>,
    // dataModelUpdate and deleteSurface parsed but not used for our renderer
}

/// Provides a flat list of component definitions for a surface.
#[derive(Debug, Deserialize)]
pub struct SurfaceUpdate {
    #[serde(rename = "surfaceId")]
    pub surface_id: String,
    pub components: Vec<A2UIComponent>,
}

/// Signals the client to start rendering from a named root component.
#[derive(Debug, Deserialize)]
pub struct BeginRendering {
    #[serde(rename = "surfaceId")]
    pub surface_id: String,
    pub root: String,
    pub styles: Option<Value>,
}

/// One entry in an A2UI flat component list.
/// `component` is an object with exactly one key (the type name, e.g. `"TextField"`).
#[derive(Debug, Deserialize)]
pub struct A2UIComponent {
    pub id: String,
    pub component: Value,
}

// ── Client-to-server output ────────────────────────────────────────────────────

/// A2UI client-to-server event wrapping a `userAction`.
#[derive(Debug, Serialize)]
pub struct A2UIOutput {
    #[serde(rename = "userAction")]
    pub user_action: UserAction,
}

/// Reports a user-initiated action.
#[derive(Debug, Serialize)]
pub struct UserAction {
    /// Action name: `"submit"` or `"cancel"`.
    pub name: String,
    #[serde(rename = "surfaceId")]
    pub surface_id: String,
    #[serde(rename = "sourceComponentId")]
    pub source_component_id: String,
    pub timestamp: String,
    /// Form field values, keyed by component id.
    pub context: HashMap<String, Value>,
}

// ── Session context (carry-over from parse to output) ─────────────────────────

pub struct A2UIContext {
    pub surface_id: String,
    pub submit_button_id: String,
    pub cancel_button_id: String,
}

// ── Detection ─────────────────────────────────────────────────────────────────

/// Returns `true` if the string appears to be A2UI JSONL format.
pub fn is_a2ui(input: &str) -> bool {
    input.contains("\"surfaceUpdate\"") || input.contains("\"beginRendering\"")
}

// ── Parsing: A2UI JSONL → A2NInput ────────────────────────────────────────────

/// Parse A2UI JSONL (one or more messages) into `(A2NInput, A2UIContext)`.
///
/// Supports:
/// - A single `{"surfaceUpdate": {...}}` object (possibly pretty-printed)
/// - Multi-line JSONL with `surfaceUpdate` and/or `beginRendering` messages
pub fn parse(json: &str) -> Result<(A2NInput, A2UIContext), String> {
    let mut all_components: Vec<(String, String, Value)> = Vec::new(); // (id, type_name, props)
    let mut surface_id = String::from("surface-1");
    let mut root_id: Option<String> = None;
    let mut styles: Option<Value> = None;

    // Use serde_json's streaming deserializer to handle both single JSON and JSONL.
    let stream = serde_json::Deserializer::from_str(json).into_iter::<A2UIMessage>();
    for msg_result in stream {
        let msg = msg_result.map_err(|e| format!("A2UI parse error: {e}"))?;

        if let Some(su) = msg.surface_update {
            surface_id = su.surface_id;
            for c in su.components {
                if let Some((tn, props)) = unwrap_component(&c.component) {
                    all_components.push((c.id, tn, props));
                }
            }
        }
        if let Some(br) = msg.begin_rendering {
            surface_id = br.surface_id;
            root_id = Some(br.root);
            styles = br.styles;
        }
    }

    // Build ID → (type, props) lookup map
    let comp_map: HashMap<String, (String, Value)> = all_components
        .iter()
        .map(|(id, t, p)| (id.clone(), (t.clone(), p.clone())))
        .collect();

    // Determine the ordered list of component IDs to render
    let ordered_ids: Vec<String> = if let Some(ref rid) = root_id {
        flatten_container(rid, &comp_map)
    } else {
        // No beginRendering — render all components in declaration order
        all_components.iter().map(|(id, _, _)| id.clone()).collect()
    };

    // Convert to our internal component types
    let components: Vec<Component> = ordered_ids
        .iter()
        .filter_map(|id| convert(id, &comp_map))
        .collect();

    // Locate submit / cancel button IDs for the output userAction
    let submit_id = find_button_id(&components, ButtonAction::Submit)
        .unwrap_or_else(|| "submit".to_string());
    let cancel_id = find_button_id(&components, ButtonAction::Cancel)
        .unwrap_or_else(|| "cancel".to_string());

    // Extract window title from the first Markdown heading component
    let title = components.iter().find_map(|c| {
        if let Component::Markdown { content, .. } = c {
            if content.starts_with('#') {
                Some(content.trim_start_matches('#').trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    });

    // Extract theme from `beginRendering.styles`
    let theme = styles.map(|s| Theme {
        dark_mode: s["darkMode"].as_bool(),
        accent_color: s["primaryColor"].as_str().map(String::from),
    });

    Ok((
        A2NInput { title, timeout: None, theme, components },
        A2UIContext { surface_id, submit_button_id: submit_id, cancel_button_id: cancel_id },
    ))
}

/// If `id` refers to a layout container (Column/Row/List), return its children
/// in order; otherwise return `[id]`.
fn flatten_container(id: &str, comp_map: &HashMap<String, (String, Value)>) -> Vec<String> {
    if let Some((type_name, props)) = comp_map.get(id) {
        match type_name.as_str() {
            "Column" | "Row" | "List" => {
                return props["children"]["explicitList"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
            }
            _ => {}
        }
    }
    vec![id.to_string()]
}

/// Extract `(type_name, props)` from an A2UI component wrapper value.
fn unwrap_component(component: &Value) -> Option<(String, Value)> {
    let obj = component.as_object()?;
    let (type_name, props) = obj.iter().next()?;
    Some((type_name.clone(), props.clone()))
}

/// Find the ID of the first button with the given action type.
fn find_button_id(components: &[Component], target: ButtonAction) -> Option<String> {
    components.iter().find_map(|c| {
        if let Component::Button { id, action, .. } = c {
            if *action == target { Some(id.clone()) } else { None }
        } else {
            None
        }
    })
}

/// Convert one A2UI component (by ID) to our internal `Component` type.
/// Returns `None` for unsupported types (Icon, Video, AudioPlayer, Tabs, Modal).
fn convert(id: &str, comp_map: &HashMap<String, (String, Value)>) -> Option<Component> {
    let (type_name, props) = comp_map.get(id)?;

    match type_name.as_str() {
        // ── Display ──────────────────────────────────────────────────────────
        "Text" => {
            let text = lit_str(&props["text"]).unwrap_or_default();
            let hint = props["usageHint"].as_str().unwrap_or("body");
            let content = match hint {
                "h1" => format!("# {text}"),
                "h2" => format!("## {text}"),
                "h3" | "h4" | "h5" => format!("### {text}"),
                _ => text.clone(),
            };
            if hint.starts_with('h') {
                Some(Component::Markdown { id: id.into(), content })
            } else {
                Some(Component::Text { id: id.into(), content })
            }
        }

        "Image" => Some(Component::Image {
            id: id.into(),
            src: lit_str(&props["url"]).unwrap_or_default(),
            alt: lit_str(&props["altText"]),
        }),

        "Divider" => Some(Component::Divider { id: id.into() }),

        // ── Action ───────────────────────────────────────────────────────────
        "Button" => {
            // The button label lives in a sibling `Text` component referenced by `child`.
            let child_id = props["child"].as_str();
            let label = child_id
                .and_then(|cid| comp_map.get(cid))
                .and_then(|(_, cp)| lit_str(&cp["text"]))
                .unwrap_or_else(|| child_id.unwrap_or("Submit").to_string());
            let action_name = props["action"]["name"].as_str().unwrap_or("submit");
            let action =
                if action_name == "cancel" { ButtonAction::Cancel } else { ButtonAction::Submit };
            Some(Component::Button { id: id.into(), label, action })
        }

        // ── Input ────────────────────────────────────────────────────────────
        "TextField" => {
            let label = lit_str(&props["label"]);
            let default_value = lit_str(&props["text"]);
            let tf_type = props["textFieldType"].as_str().unwrap_or("singleline");
            if tf_type == "multiline" {
                Some(Component::Textarea {
                    id: id.into(),
                    label,
                    placeholder: None,
                    required: false,
                    default_value,
                })
            } else {
                Some(Component::TextField {
                    id: id.into(),
                    label,
                    placeholder: None,
                    required: false,
                    default_value,
                })
            }
        }

        "CheckBox" => Some(Component::Checkbox {
            id: id.into(),
            label: lit_str(&props["label"]),
            default_value: props["value"]["literalBoolean"].as_bool().unwrap_or(false),
        }),

        "MultipleChoice" => {
            let options: Vec<SelectOption> = props["options"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|o| {
                    let value = o["value"].as_str()?.to_string();
                    let label = lit_str(&o["label"]).unwrap_or_else(|| value.clone());
                    Some(SelectOption { value, label })
                })
                .collect();
            let max = props["maxAllowedSelections"].as_i64().unwrap_or(i64::MAX);
            let variant = props["variant"].as_str().unwrap_or("list");
            if max == 1 {
                if variant == "dropdown" {
                    Some(Component::Dropdown {
                        id: id.into(),
                        label: None,
                        options,
                        required: false,
                        default_value: None,
                    })
                } else {
                    Some(Component::RadioGroup {
                        id: id.into(),
                        label: None,
                        options,
                        required: false,
                        default_value: None,
                    })
                }
            } else {
                Some(Component::CheckboxGroup {
                    id: id.into(),
                    label: None,
                    options,
                    default_values: vec![],
                })
            }
        }

        "Slider" => Some(Component::Slider {
            id: id.into(),
            label: lit_str(&props["label"]),
            min: props["minValue"].as_f64().unwrap_or(0.0),
            max: props["maxValue"].as_f64().unwrap_or(100.0),
            step: None,
            default_value: props["value"]["literalNumber"].as_f64(),
        }),

        "DateTimeInput" => {
            let enable_time = props["enableTime"].as_bool().unwrap_or(false);
            let enable_date = props["enableDate"].as_bool().unwrap_or(true);
            let default_value = lit_str(&props["value"]);
            if enable_time && !enable_date {
                Some(Component::TimePicker {
                    id: id.into(),
                    label: None,
                    required: false,
                    default_value,
                })
            } else {
                Some(Component::DatePicker {
                    id: id.into(),
                    label: None,
                    required: false,
                    default_value,
                })
            }
        }

        // ── Layout ───────────────────────────────────────────────────────────
        "Column" | "Row" | "List" => {
            let child_ids: Vec<String> = props["children"]["explicitList"]
                .as_array()
                .map(|arr| {
                    arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
                })
                .unwrap_or_default();
            let children: Vec<Component> =
                child_ids.iter().filter_map(|cid| convert(cid, comp_map)).collect();
            if children.is_empty() {
                None
            } else {
                Some(Component::Card { id: id.into(), title: None, children })
            }
        }

        "Card" => {
            let child_id = props["child"].as_str()?;
            let child = convert(child_id, comp_map)?;
            Some(Component::Card { id: id.into(), title: None, children: vec![child] })
        }

        // Icon, Video, AudioPlayer, Tabs, Modal → not supported in native renderer
        _ => None,
    }
}

/// Resolve a Google A2UI string binding `{literalString: "..."}` to a plain `String`.
/// `path` bindings are ignored since we don't have a data model.
fn lit_str(val: &Value) -> Option<String> {
    val.get("literalString")?.as_str().map(String::from)
}

// ── Output: A2NOutput → A2UIOutput ────────────────────────────────────────────

/// Convert an `A2NOutput` to an A2UI `userAction` event.
pub fn to_output(output: &A2NOutput, ctx: &A2UIContext) -> A2UIOutput {
    let (name, comp_id) = match output.status {
        OutputStatus::Submitted => ("submit", ctx.submit_button_id.as_str()),
        OutputStatus::Cancelled | OutputStatus::Timeout => {
            ("cancel", ctx.cancel_button_id.as_str())
        }
    };
    A2UIOutput {
        user_action: UserAction {
            name: name.to_string(),
            surface_id: ctx.surface_id.clone(),
            source_component_id: comp_id.to_string(),
            timestamp: now_rfc3339(),
            context: output.values.clone(),
        },
    }
}

fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (h, mi, s) = ((secs % 86400) / 3600, (secs % 3600) / 60, secs % 60);
    let (y, mo, d) = days_to_ymd(secs / 86400);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Convert a count of days since 1970-01-01 to (year, month, day).
/// Uses Howard Hinnant's algorithm: https://howardhinnant.github.io/date_algorithms.html
fn days_to_ymd(z: u64) -> (u64, u64, u64) {
    let z = z + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_a2ui_positive() {
        assert!(is_a2ui(r#"{"surfaceUpdate":{"surfaceId":"s1","components":[]}}"#));
        assert!(is_a2ui(r#"{"beginRendering":{"surfaceId":"s1","root":"r"}}"#));
    }

    #[test]
    fn test_is_a2ui_negative() {
        assert!(!is_a2ui(r#"{"components":[{"id":"f","type":"text-field"}]}"#));
    }

    #[test]
    fn test_parse_simple_form() {
        let json = r#"{"surfaceUpdate":{"surfaceId":"form1","components":[
            {"id":"lbl","component":{"Text":{"text":{"literalString":"Your name"},"usageHint":"body"}}},
            {"id":"name","component":{"TextField":{"label":{"literalString":"Name"}}}},
            {"id":"btn-lbl","component":{"Text":{"text":{"literalString":"Submit"}}}},
            {"id":"btn","component":{"Button":{"child":"btn-lbl","action":{"name":"submit"}}}}
        ]}}"#;
        let (input, ctx) = parse(json).unwrap();
        assert_eq!(ctx.surface_id, "form1");
        assert_eq!(ctx.submit_button_id, "btn");
        let has_textfield = input.components.iter().any(|c| matches!(c, Component::TextField { id, .. } if id == "name"));
        assert!(has_textfield);
        let has_button = input.components.iter().any(|c| matches!(c, Component::Button { label, .. } if label == "Submit"));
        assert!(has_button);
    }

    #[test]
    fn test_parse_with_begin_rendering() {
        // Column root + beginRendering
        let jsonl = concat!(
            r#"{"surfaceUpdate":{"surfaceId":"s","components":[{"id":"col","component":{"Column":{"children":{"explicitList":["f1","btn"]}}}},{"id":"f1","component":{"TextField":{"label":{"literalString":"Email"}}}},{"id":"btn-t","component":{"Text":{"text":{"literalString":"OK"}}}},{"id":"btn","component":{"Button":{"child":"btn-t","action":{"name":"submit"}}}}]}}"#, "\n",
            r#"{"beginRendering":{"surfaceId":"s","root":"col"}}"#
        );
        let (input, ctx) = parse(jsonl).unwrap();
        assert_eq!(ctx.surface_id, "s");
        // Should have flattened the Column into its children
        assert!(input.components.iter().any(|c| matches!(c, Component::TextField { .. })));
    }

    #[test]
    fn test_parse_multiple_choice_single() {
        let json = r#"{"surfaceUpdate":{"surfaceId":"s","components":[
            {"id":"mc","component":{"MultipleChoice":{"selections":{"literalArray":[]},"options":[{"label":{"literalString":"Red"},"value":"red"},{"label":{"literalString":"Blue"},"value":"blue"}],"maxAllowedSelections":1,"variant":"dropdown"}}}
        ]}}"#;
        let (input, _) = parse(json).unwrap();
        assert!(input.components.iter().any(|c| matches!(c, Component::Dropdown { .. })));
    }

    #[test]
    fn test_parse_multiple_choice_multi() {
        let json = r#"{"surfaceUpdate":{"surfaceId":"s","components":[
            {"id":"mc","component":{"MultipleChoice":{"selections":{"literalArray":[]},"options":[{"label":{"literalString":"A"},"value":"a"}]}}}
        ]}}"#;
        let (input, _) = parse(json).unwrap();
        assert!(input.components.iter().any(|c| matches!(c, Component::CheckboxGroup { .. })));
    }

    #[test]
    fn test_to_output_submit() {
        use std::collections::HashMap;
        let a2n_out = A2NOutput {
            status: OutputStatus::Submitted,
            values: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), serde_json::json!("Alice"));
                m
            },
        };
        let ctx = A2UIContext {
            surface_id: "form1".to_string(),
            submit_button_id: "btn-submit".to_string(),
            cancel_button_id: "btn-cancel".to_string(),
        };
        let out = to_output(&a2n_out, &ctx);
        assert_eq!(out.user_action.name, "submit");
        assert_eq!(out.user_action.surface_id, "form1");
        assert_eq!(out.user_action.source_component_id, "btn-submit");
        assert_eq!(out.user_action.context["name"], serde_json::json!("Alice"));
    }

    #[test]
    fn test_to_output_cancel() {
        use std::collections::HashMap;
        let a2n_out = A2NOutput {
            status: OutputStatus::Cancelled,
            values: HashMap::new(),
        };
        let ctx = A2UIContext {
            surface_id: "s".to_string(),
            submit_button_id: "s-btn".to_string(),
            cancel_button_id: "c-btn".to_string(),
        };
        let out = to_output(&a2n_out, &ctx);
        assert_eq!(out.user_action.name, "cancel");
        assert_eq!(out.user_action.source_component_id, "c-btn");
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_2024() {
        // 2024-01-01 = 19723 days since epoch
        assert_eq!(days_to_ymd(19723), (2024, 1, 1));
    }
}
