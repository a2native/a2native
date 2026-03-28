use eframe::egui;

use crate::protocol::{ButtonAction, Component};
use crate::renderer::egui_impl::{FormResult, FormState};

pub fn render_component(ui: &mut egui::Ui, component: &Component, state: &mut FormState) {
    match component {
        Component::Text { content, .. } => {
            ui.label(content);
        }

        Component::Markdown { content, id: _ } => {
            render_markdown(ui, content);
        }

        Component::Image { alt, src, .. } => {
            let label = alt.as_deref().unwrap_or(src.as_str());
            ui.label(format!("[image: {}]", label));
        }

        Component::Divider { .. } => {
            ui.separator();
        }

        Component::TextField { id, label, placeholder, .. } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let value = state.text_values.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("");
            ui.add(egui::TextEdit::singleline(value).hint_text(hint));
        }

        Component::Textarea { id, label, placeholder, .. } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let value = state.text_values.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("");
            ui.add(
                egui::TextEdit::multiline(value)
                    .hint_text(hint)
                    .desired_rows(4),
            );
        }

        Component::NumberInput { id, label, min, max, step, .. } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let value = state.number_values.entry(id.clone()).or_insert(0.0);
            let mut drag = egui::DragValue::new(value);
            if let (Some(lo), Some(hi)) = (min, max) {
                drag = drag.range(*lo..=*hi);
            }
            if let Some(s) = step {
                drag = drag.speed(*s);
            }
            ui.add(drag);
        }

        Component::DatePicker { id, label, .. } => {
            let lbl = label.as_deref().unwrap_or("Date");
            ui.label(format!("{} (YYYY-MM-DD):", lbl));
            let value = state.text_values.entry(id.clone()).or_default();
            ui.add(egui::TextEdit::singleline(value).hint_text("YYYY-MM-DD"));
        }

        Component::TimePicker { id, label, .. } => {
            let lbl = label.as_deref().unwrap_or("Time");
            ui.label(format!("{} (HH:MM):", lbl));
            let value = state.text_values.entry(id.clone()).or_default();
            ui.add(egui::TextEdit::singleline(value).hint_text("HH:MM"));
        }

        Component::Dropdown { id, label, options, .. } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let value = state.text_values.entry(id.clone()).or_default();
            let current = value.clone();
            let current_label = options
                .iter()
                .find(|o| o.value == current)
                .map(|o| o.label.as_str())
                .unwrap_or("-- Select --");

            egui::ComboBox::from_id_salt(id.as_str())
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    for opt in options {
                        ui.selectable_value(value, opt.value.clone(), opt.label.as_str());
                    }
                });
        }

        Component::Checkbox { id, label, .. } => {
            let checked = state.bool_values.entry(id.clone()).or_insert(false);
            let lbl = label.as_deref().unwrap_or(id.as_str());
            ui.checkbox(checked, lbl);
        }

        Component::CheckboxGroup { id, label, options, .. } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let selected = state.checkbox_group_values.entry(id.clone()).or_default();
            for opt in options {
                let mut checked = selected.contains(&opt.value);
                if ui.checkbox(&mut checked, opt.label.as_str()).changed() {
                    if checked {
                        if !selected.contains(&opt.value) {
                            selected.push(opt.value.clone());
                        }
                    } else {
                        selected.retain(|v| v != &opt.value);
                    }
                }
            }
        }

        Component::RadioGroup { id, label, options, .. } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let value = state.text_values.entry(id.clone()).or_default();
            for opt in options {
                ui.radio_value(value, opt.value.clone(), opt.label.as_str());
            }
        }

        Component::Slider { id, label, min, max, step, .. } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let value = state.number_values.entry(id.clone()).or_insert(*min);
            let mut slider = egui::Slider::new(value, *min..=*max);
            if let Some(s) = step {
                slider = slider.step_by(*s);
            }
            ui.add(slider);
        }

        Component::FileUpload { id, label, multiple, accept } => {
            if let Some(lbl) = label {
                ui.label(lbl.as_str());
            }
            let current_path = state.text_values.entry(id.clone()).or_default().clone();
            if !current_path.is_empty() {
                ui.label(format!("Selected: {}", current_path));
            }
            if ui.button("Choose File…").clicked() {
                let mut dialog = rfd::FileDialog::new();
                if let Some(filter) = accept {
                    let exts: Vec<&str> = filter
                        .split(',')
                        .map(|s| s.trim().trim_start_matches('.').trim_start_matches("*."))
                        .collect();
                    if !exts.is_empty() {
                        dialog = dialog.add_filter("Accepted files", &exts);
                    }
                }
                let picked = if *multiple {
                    dialog
                        .pick_files()
                        .map(|paths| {
                            paths
                                .iter()
                                .map(|p| p.to_string_lossy().to_string())
                                .collect::<Vec<_>>()
                                .join(";")
                        })
                } else {
                    dialog.pick_file().map(|p| p.to_string_lossy().to_string())
                };
                if let Some(path) = picked {
                    *state.text_values.entry(id.clone()).or_default() = path;
                }
            }
        }

        Component::Button { label, action, .. } => {
            if ui.button(label.as_str()).clicked() {
                match action {
                    ButtonAction::Submit => {
                        state.result = Some(FormResult::Submitted);
                    }
                    ButtonAction::Cancel => {
                        state.result = Some(FormResult::Cancelled);
                    }
                    ButtonAction::Custom => {}
                }
            }
        }

        Component::Card { id, title, children } => {
            ui.push_id(id.as_str(), |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    if let Some(t) = title {
                        ui.strong(t.as_str());
                        ui.add_space(4.0);
                    }
                    for child in children {
                        render_component(ui, child, state);
                        ui.add_space(2.0);
                    }
                });
            });
        }
    }
}

fn render_markdown(ui: &mut egui::Ui, content: &str) {
    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            ui.label(egui::RichText::new(heading).size(16.0).strong());
        } else if let Some(heading) = line.strip_prefix("## ") {
            ui.label(egui::RichText::new(heading).size(20.0).strong());
        } else if let Some(heading) = line.strip_prefix("# ") {
            ui.label(egui::RichText::new(heading).size(24.0).strong());
        } else if line.is_empty() {
            ui.add_space(4.0);
        } else {
            // Render inline bold (**text**)
            render_inline_markdown(ui, line);
        }
    }
}

fn render_inline_markdown(ui: &mut egui::Ui, line: &str) {
    // Parse **bold** segments
    ui.horizontal_wrapped(|ui| {
        let mut remaining = line;
        while !remaining.is_empty() {
            if let Some(start) = remaining.find("**") {
                let before = &remaining[..start];
                if !before.is_empty() {
                    ui.label(before);
                }
                let after_open = &remaining[start + 2..];
                if let Some(end) = after_open.find("**") {
                    let bold_text = &after_open[..end];
                    ui.label(egui::RichText::new(bold_text).strong());
                    remaining = &after_open[end + 2..];
                } else {
                    // No closing **, render rest as normal
                    ui.label(after_open);
                    break;
                }
            } else {
                ui.label(remaining);
                break;
            }
        }
    });
}


