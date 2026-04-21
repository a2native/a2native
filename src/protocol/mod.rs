use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod a2ui;
pub mod agui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2NInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark_mode: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ButtonAction {
    #[default]
    Submit,
    Cancel,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Component {
    // Display
    Text {
        id: String,
        content: String,
    },
    Markdown {
        id: String,
        content: String,
    },
    Image {
        id: String,
        src: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
    },
    Divider {
        id: String,
    },

    // Input
    TextField {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<String>,
    },
    Textarea {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<String>,
    },
    NumberInput {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<f64>,
    },
    DatePicker {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    TimePicker {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    Dropdown {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        options: Vec<SelectOption>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    Checkbox {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default)]
        default_value: bool,
    },
    CheckboxGroup {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        options: Vec<SelectOption>,
        #[serde(default)]
        default_values: Vec<String>,
    },
    RadioGroup {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        options: Vec<SelectOption>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    Slider {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default = "default_slider_min")]
        min: f64,
        #[serde(default = "default_slider_max")]
        max: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<f64>,
    },
    FileUpload {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        accept: Option<String>,
        #[serde(default)]
        multiple: bool,
    },

    // Action
    Button {
        id: String,
        label: String,
        #[serde(default)]
        action: ButtonAction,
    },

    // Layout
    Card {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        children: Vec<Component>,
    },
    Row {
        id: String,
        children: Vec<Component>,
    },

    // Extended input
    Password {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<String>,
    },
    Rating {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default = "default_rating_max")]
        max: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<u32>,
    },
    Toggle {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default)]
        default_value: bool,
    },

    // Display (extended)
    Code {
        id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },

    // ── Extended input (v0.2) ────────────────────────────────────────────────
    /// Email input — a text-field that validates as an email address on submit.
    Email {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<String>,
    },
    /// URL input — a text-field that validates as an http(s)/ftp URL on submit.
    Url {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_message: Option<String>,
    },
    /// Hidden field — never rendered. `value` is passed through verbatim to output.
    /// Useful for round-tripping opaque context (request ids, correlation tokens).
    Hidden {
        id: String,
        value: serde_json::Value,
    },
    /// Combined date + time picker. Output is a single string "YYYY-MM-DD HH:MM".
    DatetimePicker {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    /// Date range — two date fields. Output is `{ "start": "...", "end": "..." }`.
    DateRange {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_start: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_end: Option<String>,
    },
    /// Color picker — output is a hex color string "#RRGGBB".
    ColorPicker {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    /// Dual-handle range slider. Output is `{ "min": number, "max": number }`.
    RangeSlider {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default = "default_slider_min")]
        min: f64,
        #[serde(default = "default_slider_max")]
        max: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_max: Option<f64>,
    },
    /// Multi-select dropdown — pick many from a list. Output is a string array.
    MultiSelect {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        options: Vec<SelectOption>,
        #[serde(default)]
        default_values: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
    },
    /// Combobox — free-text input with a dropdown of suggestions. Output is a single string.
    Combobox {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default)]
        options: Vec<SelectOption>,
        #[serde(default)]
        required: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    /// Free-form tag input — user types a word, presses Enter/comma, it becomes a chip.
    /// Output is a string array.
    Tags {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default)]
        default_values: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
    },

    // ── Extended display (v0.2) ──────────────────────────────────────────────
    /// Alert / callout block with a severity color. No output.
    Alert {
        id: String,
        #[serde(default)]
        kind: AlertKind,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        content: String,
    },
    /// Clickable external link (opens in the system browser). No output.
    Link {
        id: String,
        label: String,
        url: String,
    },
    /// Read-only progress bar. `value` in `[0, max]`; `max` defaults to 1.0. No output.
    Progress {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        value: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(default)]
        show_percent: bool,
    },

    // ── Extended layout (v0.2) ───────────────────────────────────────────────
    /// Collapsible section — user can expand/collapse.
    Collapsible {
        id: String,
        title: String,
        #[serde(default = "default_true")]
        open: bool,
        children: Vec<Component>,
    },
    /// Explicit vertical space.
    Spacer {
        id: String,
        #[serde(default = "default_spacer_size")]
        size: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlertKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

fn default_true() -> bool { true }
fn default_spacer_size() -> f32 { 8.0 }

fn default_slider_min() -> f64 {
    0.0
}

fn default_slider_max() -> f64 {
    100.0
}

fn default_rating_max() -> u32 {
    5
}

/// IPC message sent from client to a running session daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum IpcMessage {
    /// Replace the current form with a new one.
    Update { input: A2NInput },
    /// Close the session window.
    Close,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_input() {
        let json = r#"{"components":[]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        assert!(input.title.is_none());
        assert!(input.timeout.is_none());
        assert!(input.theme.is_none());
        assert!(input.components.is_empty());
    }

    #[test]
    fn test_parse_text_field() {
        let json = r#"{
            "components": [
                {"id":"name","type":"text-field","label":"Name","placeholder":"Enter name","required":true}
            ]
        }"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.components.len(), 1);
        match &input.components[0] {
            Component::TextField { id, label, placeholder, required, .. } => {
                assert_eq!(id, "name");
                assert_eq!(label.as_deref(), Some("Name"));
                assert_eq!(placeholder.as_deref(), Some("Enter name"));
                assert!(*required);
            }
            _ => panic!("Expected TextField"),
        }
    }

    #[test]
    fn test_parse_button_submit() {
        let json = r#"{"components":[{"id":"btn","type":"button","label":"Submit","action":"submit"}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Button { action, .. } => assert_eq!(*action, ButtonAction::Submit),
            _ => panic!("Expected Button"),
        }
    }

    #[test]
    fn test_parse_button_cancel() {
        let json = r#"{"components":[{"id":"btn","type":"button","label":"Cancel","action":"cancel"}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Button { action, .. } => assert_eq!(*action, ButtonAction::Cancel),
            _ => panic!("Expected Button"),
        }
    }

    #[test]
    fn test_parse_dropdown_with_options() {
        let json = r#"{
            "components":[{
                "id":"color","type":"dropdown",
                "options":[{"value":"red","label":"Red"},{"value":"blue","label":"Blue"}]
            }]
        }"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Dropdown { id, options, .. } => {
                assert_eq!(id, "color");
                assert_eq!(options.len(), 2);
                assert_eq!(options[0].value, "red");
                assert_eq!(options[1].label, "Blue");
            }
            _ => panic!("Expected Dropdown"),
        }
    }

    #[test]
    fn test_parse_slider_defaults() {
        let json = r#"{"components":[{"id":"vol","type":"slider"}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Slider { min, max, .. } => {
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 100.0);
            }
            _ => panic!("Expected Slider"),
        }
    }

    #[test]
    fn test_parse_card_with_children() {
        let json = r#"{
            "components":[{
                "id":"section","type":"card","title":"Section",
                "children":[{"id":"field","type":"text-field","label":"Field"}]
            }]
        }"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Card { id, title, children } => {
                assert_eq!(id, "section");
                assert_eq!(title.as_deref(), Some("Section"));
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Card"),
        }
    }

    #[test]
    fn test_parse_theme() {
        let json = r##"{"components":[],"theme":{"accent_color":"#FF5733","dark_mode":true}}"##;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        let theme = input.theme.unwrap();
        assert_eq!(theme.accent_color.as_deref(), Some("#FF5733"));
        assert_eq!(theme.dark_mode, Some(true));
    }

    #[test]
    fn test_parse_timeout() {
        let json = r#"{"components":[],"timeout":30}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.timeout, Some(30));
    }

    #[test]
    fn test_output_serialization_submitted() {
        let output = A2NOutput {
            status: OutputStatus::Submitted,
            values: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), serde_json::Value::String("Alice".to_string()));
                m
            },
        };
        let json = serde_json::to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["status"], "submitted");
        assert_eq!(parsed["values"]["name"], "Alice");
    }

    #[test]
    fn test_output_serialization_cancelled() {
        let output = A2NOutput {
            status: OutputStatus::Cancelled,
            values: HashMap::new(),
        };
        let json = serde_json::to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["status"], "cancelled");
    }

    #[test]
    fn test_output_serialization_timeout() {
        let output = A2NOutput {
            status: OutputStatus::Timeout,
            values: HashMap::new(),
        };
        let json = serde_json::to_string(&output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["status"], "timeout");
    }

    #[test]
    fn test_parse_all_component_types() {
        let json = r#"{
            "components": [
                {"id":"t1","type":"text","content":"Hello"},
                {"id":"t2","type":"markdown","content":"**Bold**"},
                {"id":"t3","type":"image","src":"/img.png","alt":"img"},
                {"id":"t4","type":"divider"},
                {"id":"t5","type":"text-field"},
                {"id":"t6","type":"textarea"},
                {"id":"t7","type":"number-input"},
                {"id":"t8","type":"date-picker"},
                {"id":"t9","type":"time-picker"},
                {"id":"t10","type":"dropdown","options":[]},
                {"id":"t11","type":"checkbox"},
                {"id":"t12","type":"checkbox-group","options":[]},
                {"id":"t13","type":"radio-group","options":[]},
                {"id":"t14","type":"slider"},
                {"id":"t15","type":"file-upload"},
                {"id":"t16","type":"button","label":"Go"},
                {"id":"t17","type":"card","children":[]},
                {"id":"t18","type":"row","children":[]},
                {"id":"t19","type":"password"},
                {"id":"t20","type":"rating"},
                {"id":"t21","type":"toggle"},
                {"id":"t22","type":"code","content":"fn main() {}"},
                {"id":"t23","type":"email"},
                {"id":"t24","type":"url"},
                {"id":"t25","type":"hidden","value":"ctx-42"},
                {"id":"t26","type":"datetime-picker"},
                {"id":"t27","type":"date-range"},
                {"id":"t28","type":"color-picker"},
                {"id":"t29","type":"range-slider"},
                {"id":"t30","type":"multi-select","options":[]},
                {"id":"t31","type":"combobox"},
                {"id":"t32","type":"tags"},
                {"id":"t33","type":"alert","content":"heads up"},
                {"id":"t34","type":"link","label":"docs","url":"https://example.com"},
                {"id":"t35","type":"progress","value":0.5},
                {"id":"t36","type":"collapsible","title":"More","children":[]},
                {"id":"t37","type":"spacer"}
            ]
        }"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.components.len(), 37);
    }

    #[test]
    fn test_parse_email() {
        let json = r#"{"components":[{"id":"e","type":"email","label":"Email","required":true}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Email { id, label, required, .. } => {
                assert_eq!(id, "e");
                assert_eq!(label.as_deref(), Some("Email"));
                assert!(*required);
            }
            _ => panic!("Expected Email"),
        }
    }

    #[test]
    fn test_parse_hidden_passthrough() {
        let json = r#"{"components":[{"id":"ctx","type":"hidden","value":{"trace":"abc","n":7}}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Hidden { id, value } => {
                assert_eq!(id, "ctx");
                assert_eq!(value["trace"], "abc");
                assert_eq!(value["n"], 7);
            }
            _ => panic!("Expected Hidden"),
        }
    }

    #[test]
    fn test_parse_date_range() {
        let json = r#"{"components":[{"id":"r","type":"date-range","default_start":"2026-01-01","default_end":"2026-12-31"}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::DateRange { default_start, default_end, .. } => {
                assert_eq!(default_start.as_deref(), Some("2026-01-01"));
                assert_eq!(default_end.as_deref(), Some("2026-12-31"));
            }
            _ => panic!("Expected DateRange"),
        }
    }

    #[test]
    fn test_parse_range_slider_defaults() {
        let json = r#"{"components":[{"id":"r","type":"range-slider"}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::RangeSlider { min, max, .. } => {
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 100.0);
            }
            _ => panic!("Expected RangeSlider"),
        }
    }

    #[test]
    fn test_parse_multi_select() {
        let json = r#"{"components":[{
            "id":"m","type":"multi-select",
            "options":[{"value":"a","label":"A"},{"value":"b","label":"B"}],
            "default_values":["a"]
        }]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::MultiSelect { options, default_values, .. } => {
                assert_eq!(options.len(), 2);
                assert_eq!(default_values, &vec!["a".to_string()]);
            }
            _ => panic!("Expected MultiSelect"),
        }
    }

    #[test]
    fn test_parse_alert_kinds() {
        for (k, expected) in [
            ("info", AlertKind::Info),
            ("success", AlertKind::Success),
            ("warning", AlertKind::Warning),
            ("error", AlertKind::Error),
        ] {
            let json = format!(
                r#"{{"components":[{{"id":"a","type":"alert","kind":"{}","content":"x"}}]}}"#,
                k
            );
            let input: A2NInput = serde_json::from_str(&json).unwrap();
            match &input.components[0] {
                Component::Alert { kind, .. } => assert_eq!(*kind, expected),
                _ => panic!("Expected Alert"),
            }
        }
    }

    #[test]
    fn test_parse_collapsible_default_open() {
        let json = r#"{"components":[{"id":"c","type":"collapsible","title":"More","children":[]}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Collapsible { open, .. } => assert!(*open),
            _ => panic!("Expected Collapsible"),
        }
    }

    #[test]
    fn test_parse_tags() {
        let json = r#"{"components":[{"id":"t","type":"tags","default_values":["rust","cli"]}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Tags { default_values, .. } => {
                assert_eq!(default_values, &vec!["rust".to_string(), "cli".to_string()]);
            }
            _ => panic!("Expected Tags"),
        }
    }

    #[test]
    fn test_parse_row_with_children() {
        let json = r#"{
            "components":[{
                "id":"row1","type":"row",
                "children":[
                    {"id":"a","type":"text-field","label":"A"},
                    {"id":"b","type":"text-field","label":"B"}
                ]
            }]
        }"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Row { id, children } => {
                assert_eq!(id, "row1");
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected Row"),
        }
    }

    #[test]
    fn test_parse_rating_defaults() {
        let json = r#"{"components":[{"id":"r1","type":"rating"}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Rating { max, default_value, .. } => {
                assert_eq!(*max, 5);
                assert!(default_value.is_none());
            }
            _ => panic!("Expected Rating"),
        }
    }

    #[test]
    fn test_parse_password() {
        let json = r#"{"components":[{"id":"pw","type":"password","label":"Password","required":true}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Password { id, label, required, .. } => {
                assert_eq!(id, "pw");
                assert_eq!(label.as_deref(), Some("Password"));
                assert!(*required);
            }
            _ => panic!("Expected Password"),
        }
    }

    #[test]
    fn test_parse_code() {
        let json = r#"{"components":[{"id":"c1","type":"code","content":"hello","language":"rust"}]}"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        match &input.components[0] {
            Component::Code { content, language, .. } => {
                assert_eq!(content, "hello");
                assert_eq!(language.as_deref(), Some("rust"));
            }
            _ => panic!("Expected Code"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2NOutput {
    pub status: OutputStatus,
    pub values: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStatus {
    Submitted,
    Cancelled,
    Timeout,
}
