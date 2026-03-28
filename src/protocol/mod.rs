use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
