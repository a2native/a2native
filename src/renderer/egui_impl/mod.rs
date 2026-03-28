pub mod components;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui;

use crate::protocol::{A2NInput, A2NOutput, ButtonAction, Component, OutputStatus};
use crate::renderer::Renderer;

pub struct FormState {
    pub text_values: HashMap<String, String>,
    pub number_values: HashMap<String, f64>,
    pub bool_values: HashMap<String, bool>,
    pub checkbox_group_values: HashMap<String, Vec<String>>,
    pub result: Option<FormResult>,
}

pub enum FormResult {
    Submitted,
    Cancelled,
    Timeout,
}

impl FormState {
    pub fn from_input(input: &A2NInput) -> Self {
        let mut state = FormState {
            text_values: HashMap::new(),
            number_values: HashMap::new(),
            bool_values: HashMap::new(),
            checkbox_group_values: HashMap::new(),
            result: None,
        };
        Self::init_from_components(&mut state, &input.components);
        state
    }

    fn init_from_components(state: &mut FormState, components: &[Component]) {
        for component in components {
            match component {
                Component::TextField { id, default_value, .. } => {
                    state.text_values.insert(
                        id.clone(),
                        default_value.clone().unwrap_or_default(),
                    );
                }
                Component::Textarea { id, default_value, .. } => {
                    state.text_values.insert(
                        id.clone(),
                        default_value.clone().unwrap_or_default(),
                    );
                }
                Component::NumberInput { id, default_value, .. } => {
                    state.number_values.insert(id.clone(), default_value.unwrap_or(0.0));
                }
                Component::DatePicker { id, default_value, .. } => {
                    state.text_values.insert(
                        id.clone(),
                        default_value.clone().unwrap_or_default(),
                    );
                }
                Component::TimePicker { id, default_value, .. } => {
                    state.text_values.insert(
                        id.clone(),
                        default_value.clone().unwrap_or_default(),
                    );
                }
                Component::Dropdown { id, default_value, .. } => {
                    state.text_values.insert(
                        id.clone(),
                        default_value.clone().unwrap_or_default(),
                    );
                }
                Component::Checkbox { id, default_value, .. } => {
                    state.bool_values.insert(id.clone(), *default_value);
                }
                Component::CheckboxGroup { id, default_values, .. } => {
                    state.checkbox_group_values.insert(id.clone(), default_values.clone());
                }
                Component::RadioGroup { id, default_value, .. } => {
                    state.text_values.insert(
                        id.clone(),
                        default_value.clone().unwrap_or_default(),
                    );
                }
                Component::Slider { id, default_value, min, .. } => {
                    state.number_values.insert(id.clone(), default_value.unwrap_or(*min));
                }
                Component::FileUpload { id, .. } => {
                    state.text_values.insert(id.clone(), String::new());
                }
                Component::Card { children, .. } => {
                    Self::init_from_components(state, children);
                }
                _ => {}
            }
        }
    }

    pub fn collect_output(&self, components: &[Component]) -> HashMap<String, serde_json::Value> {
        let mut values = HashMap::new();
        self.collect_from_components(&mut values, components);
        values
    }

    fn collect_from_components(
        &self,
        values: &mut HashMap<String, serde_json::Value>,
        components: &[Component],
    ) {
        for component in components {
            match component {
                Component::TextField { id, .. }
                | Component::Textarea { id, .. }
                | Component::DatePicker { id, .. }
                | Component::TimePicker { id, .. }
                | Component::Dropdown { id, .. }
                | Component::RadioGroup { id, .. }
                | Component::FileUpload { id, .. } => {
                    if let Some(v) = self.text_values.get(id) {
                        values.insert(id.clone(), serde_json::Value::String(v.clone()));
                    }
                }
                Component::NumberInput { id, .. } | Component::Slider { id, .. } => {
                    if let Some(v) = self.number_values.get(id) {
                        values.insert(
                            id.clone(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(*v)
                                    .unwrap_or(serde_json::Number::from(0)),
                            ),
                        );
                    }
                }
                Component::Checkbox { id, .. } => {
                    if let Some(v) = self.bool_values.get(id) {
                        values.insert(id.clone(), serde_json::Value::Bool(*v));
                    }
                }
                Component::CheckboxGroup { id, .. } => {
                    if let Some(v) = self.checkbox_group_values.get(id) {
                        values.insert(
                            id.clone(),
                            serde_json::Value::Array(
                                v.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
                            ),
                        );
                    }
                }
                Component::Card { children, .. } => {
                    self.collect_from_components(values, children);
                }
                _ => {}
            }
        }
    }
}

pub struct EguiRenderer;

impl EguiRenderer {
    pub fn new() -> Self {
        EguiRenderer
    }
}

impl Default for EguiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for EguiRenderer {
    fn run(self, input: A2NInput) -> A2NOutput {
        let title = input.title.clone().unwrap_or_else(|| "A2N Form".to_string());
        let output_slot: Arc<Mutex<Option<A2NOutput>>> = Arc::new(Mutex::new(None));
        let output_slot_clone = Arc::clone(&output_slot);

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(&title)
                .with_inner_size([600.0, 500.0])
                .with_resizable(true),
            ..Default::default()
        };

        let app = A2NApp::new(input, output_slot_clone);

        let _ = eframe::run_native(
            &title,
            native_options,
            Box::new(move |_cc| Ok(Box::new(app))),
        );

        let output = output_slot.lock().unwrap().take();
        output.unwrap_or_else(|| A2NOutput {
            status: OutputStatus::Cancelled,
            values: HashMap::new(),
        })
    }
}

struct A2NApp {
    input: A2NInput,
    state: FormState,
    output_slot: Arc<Mutex<Option<A2NOutput>>>,
    start_time: Instant,
    has_submit_button: bool,
}

impl A2NApp {
    fn new(input: A2NInput, output_slot: Arc<Mutex<Option<A2NOutput>>>) -> Self {
        let state = FormState::from_input(&input);
        let has_submit_button = Self::check_has_submit(&input.components);
        A2NApp {
            input,
            state,
            output_slot,
            start_time: Instant::now(),
            has_submit_button,
        }
    }

    fn check_has_submit(components: &[Component]) -> bool {
        for c in components {
            match c {
                Component::Button { action, .. } if *action == ButtonAction::Submit => return true,
                Component::Card { children, .. } => {
                    if Self::check_has_submit(children) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        if let Some(theme) = &self.input.theme {
            let mut visuals = match theme.dark_mode {
                Some(true) => egui::Visuals::dark(),
                Some(false) => egui::Visuals::light(),
                None => ctx.style().visuals.clone(),
            };

            if let Some(color_str) = &theme.accent_color {
                if let Some(color) = parse_hex_color(color_str) {
                    visuals.selection.bg_fill = color;
                    visuals.hyperlink_color = color;
                }
            }

            ctx.set_visuals(visuals);
        }
    }

    fn finalize(&mut self, ctx: &egui::Context, result: FormResult) {
        let status = match result {
            FormResult::Submitted => OutputStatus::Submitted,
            FormResult::Cancelled => OutputStatus::Cancelled,
            FormResult::Timeout => OutputStatus::Timeout,
        };
        let values = self.state.collect_output(&self.input.components);
        let output = A2NOutput { status, values };
        *self.output_slot.lock().unwrap() = Some(output);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for A2NApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check timeout
        if let Some(timeout_secs) = self.input.timeout {
            if self.start_time.elapsed().as_secs() >= timeout_secs {
                self.finalize(ctx, FormResult::Timeout);
                return;
            }
        }

        self.apply_theme(ctx);

        // Check if we should close due to a result being set in state
        if self.state.result.is_some() {
            let result = self.state.result.take().unwrap();
            self.finalize(ctx, result);
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(title) = &self.input.title.clone() {
                    ui.heading(title);
                    ui.add_space(8.0);
                }

                let components = self.input.components.clone();
                for component in &components {
                    components::render_component(ui, component, &mut self.state);
                    ui.add_space(4.0);
                }

                if !self.has_submit_button {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                    if ui.button("Submit").clicked() {
                        self.state.result = Some(FormResult::Submitted);
                    }
                }
            });
        });
    }
}

pub fn parse_hex_color(hex: &str) -> Option<egui::Color32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(egui::Color32::from_rgb(r, g, b))
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        Some(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{A2NInput, Component, SelectOption};

    fn make_input(components: Vec<Component>) -> A2NInput {
        A2NInput {
            title: None,
            timeout: None,
            theme: None,
            components,
        }
    }

    #[test]
    fn test_parse_hex_color_6digit() {
        let color = parse_hex_color("#FF5733").unwrap();
        assert_eq!(color, egui::Color32::from_rgb(0xFF, 0x57, 0x33));
    }

    #[test]
    fn test_parse_hex_color_without_hash() {
        let color = parse_hex_color("00FF00").unwrap();
        assert_eq!(color, egui::Color32::from_rgb(0x00, 0xFF, 0x00));
    }

    #[test]
    fn test_parse_hex_color_8digit() {
        let color = parse_hex_color("#FF573380").unwrap();
        assert_eq!(color, egui::Color32::from_rgba_unmultiplied(0xFF, 0x57, 0x33, 0x80));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert!(parse_hex_color("ZZZZZZ").is_none());
        assert!(parse_hex_color("123").is_none());
        assert!(parse_hex_color("").is_none());
    }

    #[test]
    fn test_form_state_init_text_defaults() {
        let input = make_input(vec![
            Component::TextField {
                id: "name".to_string(),
                label: None,
                placeholder: None,
                required: false,
                default_value: Some("Alice".to_string()),
            },
        ]);
        let state = FormState::from_input(&input);
        assert_eq!(state.text_values.get("name").map(|s| s.as_str()), Some("Alice"));
    }

    #[test]
    fn test_form_state_init_number_defaults() {
        let input = make_input(vec![
            Component::NumberInput {
                id: "age".to_string(),
                label: None,
                min: None,
                max: None,
                step: None,
                default_value: Some(25.0),
            },
        ]);
        let state = FormState::from_input(&input);
        assert_eq!(state.number_values.get("age"), Some(&25.0));
    }

    #[test]
    fn test_form_state_init_checkbox() {
        let input = make_input(vec![
            Component::Checkbox {
                id: "agree".to_string(),
                label: None,
                default_value: true,
            },
        ]);
        let state = FormState::from_input(&input);
        assert_eq!(state.bool_values.get("agree"), Some(&true));
    }

    #[test]
    fn test_form_state_init_checkbox_group() {
        let input = make_input(vec![
            Component::CheckboxGroup {
                id: "tags".to_string(),
                label: None,
                options: vec![
                    SelectOption { value: "a".to_string(), label: "A".to_string() },
                    SelectOption { value: "b".to_string(), label: "B".to_string() },
                ],
                default_values: vec!["a".to_string()],
            },
        ]);
        let state = FormState::from_input(&input);
        assert_eq!(state.checkbox_group_values.get("tags"), Some(&vec!["a".to_string()]));
    }

    #[test]
    fn test_form_state_init_slider_uses_min_as_default() {
        let input = make_input(vec![
            Component::Slider {
                id: "vol".to_string(),
                label: None,
                min: 10.0,
                max: 50.0,
                step: None,
                default_value: None,
            },
        ]);
        let state = FormState::from_input(&input);
        assert_eq!(state.number_values.get("vol"), Some(&10.0));
    }

    #[test]
    fn test_form_state_collect_output() {
        let components = vec![
            Component::TextField {
                id: "name".to_string(),
                label: None,
                placeholder: None,
                required: false,
                default_value: None,
            },
            Component::Checkbox {
                id: "ok".to_string(),
                label: None,
                default_value: false,
            },
            Component::NumberInput {
                id: "count".to_string(),
                label: None,
                min: None,
                max: None,
                step: None,
                default_value: Some(3.0),
            },
        ];
        let input = make_input(components.clone());
        let mut state = FormState::from_input(&input);
        *state.text_values.get_mut("name").unwrap() = "Bob".to_string();
        *state.bool_values.get_mut("ok").unwrap() = true;

        let values = state.collect_output(&components);
        assert_eq!(values.get("name"), Some(&serde_json::Value::String("Bob".to_string())));
        assert_eq!(values.get("ok"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(values.get("count"), Some(&serde_json::json!(3.0)));
    }

    #[test]
    fn test_check_has_submit_true() {
        let components = vec![
            Component::Button {
                id: "s".to_string(),
                label: "Submit".to_string(),
                action: ButtonAction::Submit,
            },
        ];
        let input = make_input(components);
        let app_has_submit = A2NApp::check_has_submit(&input.components);
        assert!(app_has_submit);
    }

    #[test]
    fn test_check_has_submit_false() {
        let components = vec![
            Component::Text { id: "t".to_string(), content: "hello".to_string() },
        ];
        let input = make_input(components);
        assert!(!A2NApp::check_has_submit(&input.components));
    }

    #[test]
    fn test_check_has_submit_in_card() {
        let components = vec![
            Component::Card {
                id: "c".to_string(),
                title: None,
                children: vec![Component::Button {
                    id: "b".to_string(),
                    label: "Go".to_string(),
                    action: ButtonAction::Submit,
                }],
            },
        ];
        let input = make_input(components);
        assert!(A2NApp::check_has_submit(&input.components));
    }
}
