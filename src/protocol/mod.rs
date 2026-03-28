use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod a2ui;

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
}

fn default_slider_min() -> f64 {
    0.0
}

fn default_slider_max() -> f64 {
    100.0
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
                {"id":"t17","type":"card","children":[]}
            ]
        }"#;
        let input: A2NInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.components.len(), 17);
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
