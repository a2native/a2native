use eframe::egui;

use crate::protocol::{AlertKind, ButtonAction, Component};
use crate::renderer::egui_impl::{FormResult, FormState};

const BTN_PRIMARY_BG:   egui::Color32 = egui::Color32::from_rgb(52, 120, 246);
const BTN_PRIMARY_TEXT: egui::Color32 = egui::Color32::WHITE;
const ERR_COLOR: egui::Color32 = egui::Color32::from_rgb(220, 60, 60);

/// Render a form field label in a consistent style.
fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).strong());
}

pub fn render_component(ui: &mut egui::Ui, component: &Component, state: &mut FormState) {
    match component {
        Component::Text { content, .. } => {
            ui.label(content);
        }

        Component::Markdown { content, id: _ } => {
            render_markdown(ui, content);
        }

        Component::Image { src, alt, .. } => {
            if !state.image_cache.contains_key(src.as_str()) {
                let texture = load_image_as_texture(ui.ctx(), src);
                state.image_cache.insert(src.clone(), texture);
            }
            match state.image_cache.get(src.as_str()).and_then(|t| t.as_ref()) {
                Some(texture) => {
                    let size = texture.size_vec2();
                    let tex_id = texture.id();
                    let max_width = ui.available_width();
                    let scale = if size.x > max_width { max_width / size.x } else { 1.0 };
                    let display_size = egui::vec2(size.x * scale, size.y * scale);
                    ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, display_size)));
                }
                None => {
                    let label = alt.as_deref().unwrap_or(src.as_str());
                    ui.label(egui::RichText::new(format!("[image: {}]", label)).weak());
                }
            }
        }

        Component::Divider { .. } => {
            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);
        }

        Component::TextField { id, label, placeholder, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("");
            let response = ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
            if response.changed() {
                state.validation_errors.remove(id.as_str());
            }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Textarea { id, label, placeholder, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("");
            let response = ui.add(
                egui::TextEdit::multiline(value)
                    .hint_text(hint)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            if response.changed() {
                state.validation_errors.remove(id.as_str());
            }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::NumberInput { id, label, min, max, step, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.number_values.entry(id.clone()).or_insert(0.0);
            let mut drag = egui::DragValue::new(value).speed(0.1);
            if let (Some(lo), Some(hi)) = (min, max) {
                drag = drag.range(*lo..=*hi);
            }
            if let Some(s) = step {
                drag = drag.speed(*s);
            }
            ui.add(drag);
        }

        Component::DatePicker { id, label, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            else { field_label(ui, "Date"); }
            let value = state.text_values.entry(id.clone()).or_default();
            let response = ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(f32::INFINITY),
            );
            if response.changed() {
                state.validation_errors.remove(id.as_str());
            }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::TimePicker { id, label, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            else { field_label(ui, "Time"); }
            let value = state.text_values.entry(id.clone()).or_default();
            let response = ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text("HH:MM")
                    .desired_width(f32::INFINITY),
            );
            if response.changed() {
                state.validation_errors.remove(id.as_str());
            }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Dropdown { id, label, options, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            let current_label = options
                .iter()
                .find(|o| o.value == *value)
                .map(|o| o.label.as_str())
                .unwrap_or("-- Select --");

            egui::ComboBox::from_id_salt(id.as_str())
                .selected_text(current_label)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for opt in options {
                        if ui.selectable_value(value, opt.value.clone(), opt.label.as_str()).changed() {
                            state.validation_errors.remove(id.as_str());
                        }
                    }
                });
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Checkbox { id, label, .. } => {
            ui.add_space(2.0);
            let checked = state.bool_values.entry(id.clone()).or_insert(false);
            let lbl = label.as_deref().unwrap_or(id.as_str());
            ui.checkbox(checked, lbl);
        }

        Component::CheckboxGroup { id, label, options, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
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
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            for opt in options {
                if ui.radio_value(value, opt.value.clone(), opt.label.as_str()).changed() {
                    state.validation_errors.remove(id.as_str());
                }
            }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Slider { id, label, min, max, step, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.number_values.entry(id.clone()).or_insert(*min);
            let mut slider = egui::Slider::new(value, *min..=*max).show_value(true);
            if let Some(s) = step {
                slider = slider.step_by(*s);
            }
            ui.add(slider);
        }

        Component::FileUpload { id, label, multiple, accept } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let current_path = state.text_values.entry(id.clone()).or_default().clone();
            ui.horizontal(|ui| {
                if !current_path.is_empty() {
                    ui.label(egui::RichText::new(&current_path).weak().small());
                }
                if ui.button("  Choose File…  ").clicked() {
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
                        dialog.pick_files().map(|paths| {
                            paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>().join(";")
                        })
                    } else {
                        dialog.pick_file().map(|p| p.to_string_lossy().to_string())
                    };
                    if let Some(path) = picked {
                        *state.text_values.entry(id.clone()).or_default() = path;
                    }
                }
            });
        }

        Component::Button { label, action, .. } => {
            ui.add_space(2.0);
            let response = match action {
                ButtonAction::Submit => ui.add(
                    egui::Button::new(
                        egui::RichText::new(label).strong().color(BTN_PRIMARY_TEXT),
                    )
                    .fill(BTN_PRIMARY_BG)
                    .min_size(egui::vec2(90.0, 32.0)),
                ),
                _ => ui.add(egui::Button::new(label.as_str()).min_size(egui::vec2(80.0, 32.0))),
            };
            if response.clicked() {
                match action {
                    ButtonAction::Submit => state.result = Some(FormResult::Submitted),
                    ButtonAction::Cancel => state.result = Some(FormResult::Cancelled),
                    ButtonAction::Custom => {}
                }
            }
        }

        Component::Card { id, title, children } => {
            ui.push_id(id.as_str(), |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                    .show(ui, |ui| {
                        if let Some(t) = title {
                            ui.label(egui::RichText::new(t).strong());
                            ui.add_space(6.0);
                        }
                        for child in children {
                            render_component(ui, child, state);
                            ui.add_space(4.0);
                        }
                    });
            });
        }

        Component::Row { id, children } => {
            ui.push_id(id.as_str(), |ui| {
                let n = children.len();
                if n > 0 {
                    ui.columns(n, |cols| {
                        for (i, child) in children.iter().enumerate() {
                            render_component(&mut cols[i], child, state);
                        }
                    });
                }
            });
        }

        Component::Password { id, label, placeholder, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("");
            let response = ui.add(
                egui::TextEdit::singleline(value)
                    .password(true)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
            if response.changed() {
                state.validation_errors.remove(id.as_str());
            }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Rating { id, label, max, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let current = state.number_values.entry(id.clone()).or_insert(0.0);
            let max_stars = *max as usize;
            ui.horizontal(|ui| {
                for i in 1..=max_stars {
                    let filled = (*current as usize) >= i;
                    let star = if filled { "★" } else { "☆" };
                    let color = if filled {
                        egui::Color32::from_rgb(255, 180, 0)
                    } else {
                        egui::Color32::GRAY
                    };
                    let resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new(star).color(color).size(24.0),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if resp.clicked() {
                        *current = i as f64;
                    }
                }
            });
        }

        Component::Toggle { id, label, .. } => {
            ui.horizontal(|ui| {
                let lbl = label.as_deref().unwrap_or(id.as_str());
                if !lbl.is_empty() {
                    ui.label(lbl);
                }
                let checked = state.bool_values.entry(id.clone()).or_insert(false);
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(44.0, 22.0), egui::Sense::click());
                if resp.clicked() {
                    *checked = !*checked;
                }
                let bg = if *checked {
                    egui::Color32::from_rgb(52, 120, 246)
                } else {
                    egui::Color32::from_rgb(160, 160, 160)
                };
                ui.painter().rect_filled(rect, egui::Rounding::same(11.0), bg);
                let cx = if *checked { rect.max.x - 11.0 } else { rect.min.x + 11.0 };
                ui.painter()
                    .circle_filled(egui::pos2(cx, rect.center().y), 8.5, egui::Color32::WHITE);
            });
        }

        Component::Code { content, .. } => {
            let frame = egui::Frame {
                fill: egui::Color32::from_gray(30),
                inner_margin: egui::Margin::symmetric(10.0, 8.0),
                rounding: egui::Rounding::same(4.0),
                ..Default::default()
            };
            frame.show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(content.as_str())
                            .monospace()
                            .color(egui::Color32::from_rgb(180, 220, 180))
                            .size(12.0),
                    )
                    .wrap_mode(egui::TextWrapMode::Extend),
                );
            });
        }

        // ── v0.2 additions ───────────────────────────────────────────────────
        Component::Email { id, label, placeholder, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("you@example.com");
            let resp = ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
            if resp.changed() { state.validation_errors.remove(id.as_str()); }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Url { id, label, placeholder, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("https://…");
            let resp = ui.add(
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
            if resp.changed() { state.validation_errors.remove(id.as_str()); }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Hidden { .. } => {
            // Intentionally not rendered — value is passed through to output.
        }

        Component::DatetimePicker { id, label, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let (date, time) = state
                .pair_text_values
                .entry(id.clone())
                .or_insert_with(|| (String::new(), String::new()));
            let mut changed = false;
            ui.horizontal(|ui| {
                let r1 = ui.add(
                    egui::TextEdit::singleline(date)
                        .hint_text("YYYY-MM-DD")
                        .desired_width(140.0),
                );
                let r2 = ui.add(
                    egui::TextEdit::singleline(time)
                        .hint_text("HH:MM")
                        .desired_width(80.0),
                );
                changed = r1.changed() || r2.changed();
            });
            if changed { state.validation_errors.remove(id.as_str()); }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::DateRange { id, label, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let (start, end) = state
                .pair_text_values
                .entry(id.clone())
                .or_insert_with(|| (String::new(), String::new()));
            let mut changed = false;
            ui.horizontal(|ui| {
                let r1 = ui.add(
                    egui::TextEdit::singleline(start)
                        .hint_text("Start YYYY-MM-DD")
                        .desired_width(140.0),
                );
                ui.label("→");
                let r2 = ui.add(
                    egui::TextEdit::singleline(end)
                        .hint_text("End YYYY-MM-DD")
                        .desired_width(140.0),
                );
                changed = r1.changed() || r2.changed();
            });
            if changed { state.validation_errors.remove(id.as_str()); }
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::ColorPicker { id, label, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let hex = state
                .text_values
                .entry(id.clone())
                .or_insert_with(|| "#6C63FF".to_string());
            let mut rgb = hex_to_rgb(hex.as_str()).unwrap_or([108, 99, 255]);
            ui.horizontal(|ui| {
                if ui.color_edit_button_srgb(&mut rgb).changed() {
                    *hex = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                }
                ui.label(egui::RichText::new(hex.as_str()).monospace().small());
            });
        }

        Component::RangeSlider { id, label, min, max, step, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let (lo, hi) = state
                .pair_number_values
                .entry(id.clone())
                .or_insert((*min, *max));
            ui.horizontal(|ui| {
                let mut s_lo = egui::Slider::new(lo, *min..=*max).show_value(true).text("min");
                let mut s_hi = egui::Slider::new(hi, *min..=*max).show_value(true).text("max");
                if let Some(sp) = step {
                    s_lo = s_lo.step_by(*sp);
                    s_hi = s_hi.step_by(*sp);
                }
                ui.add(s_lo);
                ui.add(s_hi);
            });
            // Keep lo ≤ hi.
            if *lo > *hi { std::mem::swap(lo, hi); }
        }

        Component::MultiSelect { id, label, options, placeholder, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let selected = state.checkbox_group_values.entry(id.clone()).or_default();
            let summary = if selected.is_empty() {
                placeholder.clone().unwrap_or_else(|| "-- Select --".to_string())
            } else {
                let labels: Vec<&str> = options
                    .iter()
                    .filter(|o| selected.contains(&o.value))
                    .map(|o| o.label.as_str())
                    .collect();
                if labels.len() <= 3 {
                    labels.join(", ")
                } else {
                    format!("{} selected", labels.len())
                }
            };
            egui::ComboBox::from_id_salt(id.as_str())
                .selected_text(summary)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
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
                });
        }

        Component::Combobox { id, label, placeholder, options, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let value = state.text_values.entry(id.clone()).or_default();
            ui.horizontal(|ui| {
                let hint = placeholder.as_deref().unwrap_or("");
                let resp = ui.add(
                    egui::TextEdit::singleline(value)
                        .hint_text(hint)
                        .desired_width(ui.available_width() - 30.0),
                );
                if resp.changed() { state.validation_errors.remove(id.as_str()); }
                egui::ComboBox::from_id_salt(format!("{}__suggest", id))
                    .selected_text("▾")
                    .width(28.0)
                    .show_ui(ui, |ui| {
                        for opt in options {
                            if ui
                                .selectable_label(*value == opt.value, opt.label.as_str())
                                .clicked()
                            {
                                *value = opt.value.clone();
                                state.validation_errors.remove(id.as_str());
                                ui.close_menu();
                            }
                        }
                    });
            });
            if let Some(err) = state.validation_errors.get(id.as_str()) {
                ui.label(egui::RichText::new(err).color(ERR_COLOR).small());
            }
        }

        Component::Tags { id, label, placeholder, max, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let tags = state.checkbox_group_values.entry(id.clone()).or_default();
            // Chips row
            let mut remove_idx: Option<usize> = None;
            ui.horizontal_wrapped(|ui| {
                for (i, tag) in tags.iter().enumerate() {
                    let chip = egui::Frame {
                        fill: egui::Color32::from_rgb(52, 120, 246).linear_multiply(0.2),
                        inner_margin: egui::Margin::symmetric(8.0, 3.0),
                        rounding: egui::Rounding::same(10.0),
                        ..Default::default()
                    };
                    chip.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(tag).small());
                            if ui
                                .add(egui::Label::new(
                                    egui::RichText::new("×").small().color(egui::Color32::GRAY),
                                ).sense(egui::Sense::click()))
                                .clicked()
                            {
                                remove_idx = Some(i);
                            }
                        });
                    });
                }
            });
            if let Some(i) = remove_idx { tags.remove(i); }

            // Input buffer — Enter or comma commits.
            let buffer = state.tags_input_buffer.entry(id.clone()).or_default();
            let hint = placeholder.as_deref().unwrap_or("Type a tag, press Enter…");
            let resp = ui.add(
                egui::TextEdit::singleline(buffer)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let has_comma = buffer.contains(',');
            if enter || has_comma {
                // Split on commas to support bulk paste.
                let pieces: Vec<String> = buffer
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                for p in pieces {
                    if let Some(limit) = max {
                        if tags.len() >= *limit { break; }
                    }
                    if !tags.contains(&p) {
                        tags.push(p);
                    }
                }
                buffer.clear();
                if enter { resp.request_focus(); }
            }
        }

        Component::Alert { kind, title, content, .. } => {
            let (bg, border, icon) = match kind {
                AlertKind::Info    => (egui::Color32::from_rgb(220, 235, 255), egui::Color32::from_rgb(70, 130, 220),  "ℹ"),
                AlertKind::Success => (egui::Color32::from_rgb(220, 245, 225), egui::Color32::from_rgb(55, 160, 90),   "✔"),
                AlertKind::Warning => (egui::Color32::from_rgb(255, 245, 210), egui::Color32::from_rgb(210, 150, 40),  "⚠"),
                AlertKind::Error   => (egui::Color32::from_rgb(255, 225, 225), egui::Color32::from_rgb(210, 70, 70),   "✖"),
            };
            let frame = egui::Frame {
                fill: bg,
                inner_margin: egui::Margin::symmetric(10.0, 8.0),
                rounding: egui::Rounding::same(6.0),
                stroke: egui::Stroke::new(1.0, border),
                ..Default::default()
            };
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).color(border).size(16.0));
                    ui.vertical(|ui| {
                        if let Some(t) = title {
                            ui.label(
                                egui::RichText::new(t)
                                    .strong()
                                    .color(egui::Color32::from_gray(20)),
                            );
                        }
                        ui.label(
                            egui::RichText::new(content.as_str())
                                .color(egui::Color32::from_gray(30)),
                        );
                    });
                });
            });
        }

        Component::Link { label, url, .. } => {
            ui.hyperlink_to(label.as_str(), url.as_str());
        }

        Component::Progress { label, value, max, show_percent, .. } => {
            if let Some(lbl) = label { field_label(ui, lbl); }
            let m = max.unwrap_or(1.0).max(f64::EPSILON);
            let frac = (*value / m).clamp(0.0, 1.0) as f32;
            let text = if *show_percent {
                format!("{:.0}%", frac * 100.0)
            } else {
                format!("{:.0} / {:.0}", value, m)
            };
            ui.add(egui::ProgressBar::new(frac).text(text));
        }

        Component::Collapsible { id, title, open, children } => {
            let initial = *state.collapsible_open.entry(id.clone()).or_insert(*open);
            let resp = egui::CollapsingHeader::new(title.as_str())
                .id_salt(id.as_str())
                .default_open(initial)
                .show(ui, |ui| {
                    for child in children {
                        render_component(ui, child, state);
                        ui.add_space(4.0);
                    }
                });
            state.collapsible_open.insert(id.clone(), resp.fully_open());
        }

        Component::Spacer { size, .. } => {
            ui.add_space(*size);
        }
    }
}

fn hex_to_rgb(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    Some([
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ])
}

fn render_markdown(ui: &mut egui::Ui, content: &str) {
    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            ui.label(egui::RichText::new(heading).size(16.0).strong());
        } else if let Some(heading) = line.strip_prefix("## ") {
            ui.label(egui::RichText::new(heading).size(20.0).strong());
        } else if let Some(heading) = line.strip_prefix("# ") {
            ui.label(egui::RichText::new(heading).size(24.0).strong());
        } else if line == "---" || line == "***" || line == "___" {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
        } else if line.is_empty() {
            ui.add_space(4.0);
        } else if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            ui.horizontal_wrapped(|ui| {
                ui.label("•");
                render_inline_markdown(ui, item);
            });
        } else if let Some(rest) = strip_numbered_list(line) {
            let dot_pos = line.find(". ").unwrap();
            let num = &line[..dot_pos + 1];
            ui.horizontal_wrapped(|ui| {
                ui.label(num);
                render_inline_markdown(ui, rest);
            });
        } else {
            render_inline_markdown(ui, line);
        }
    }
}

fn strip_numbered_list(line: &str) -> Option<&str> {
    let mut idx = 0;
    let mut has_digit = false;
    for ch in line.chars() {
        if ch.is_ascii_digit() {
            has_digit = true;
            idx += ch.len_utf8();
        } else if ch == '.' && has_digit {
            idx += ch.len_utf8();
            if line[idx..].starts_with(' ') {
                return Some(&line[idx + 1..]);
            }
            return None;
        } else {
            return None;
        }
    }
    None
}

fn render_inline_markdown(ui: &mut egui::Ui, line: &str) {
    ui.horizontal_wrapped(|ui| {
        let mut remaining = line;
        while !remaining.is_empty() {
            // Try **bold** first
            if let Some(start) = remaining.find("**") {
                if start > 0 {
                    render_inline_rest(ui, &remaining[..start]);
                }
                let after_open = &remaining[start + 2..];
                if let Some(end) = after_open.find("**") {
                    ui.label(egui::RichText::new(&after_open[..end]).strong());
                    remaining = &after_open[end + 2..];
                    continue;
                } else {
                    render_inline_rest(ui, remaining);
                    break;
                }
            }
            // Try `code`
            if let Some(start) = remaining.find('`') {
                if start > 0 {
                    render_inline_rest(ui, &remaining[..start]);
                }
                let after_open = &remaining[start + 1..];
                if let Some(end) = after_open.find('`') {
                    ui.label(egui::RichText::new(&after_open[..end]).monospace().size(12.0));
                    remaining = &after_open[end + 1..];
                    continue;
                } else {
                    render_inline_rest(ui, after_open);
                    break;
                }
            }
            // Try [link](url)
            if let Some(bracket_start) = remaining.find('[') {
                if bracket_start > 0 {
                    render_inline_rest(ui, &remaining[..bracket_start]);
                }
                let after_bracket = &remaining[bracket_start + 1..];
                if let Some(bracket_end) = after_bracket.find(']') {
                    let link_text = &after_bracket[..bracket_end];
                    let after_close = &after_bracket[bracket_end + 1..];
                    if after_close.starts_with('(') {
                        if let Some(paren_end) = after_close.find(')') {
                            ui.label(
                                egui::RichText::new(link_text)
                                    .color(egui::Color32::from_rgb(80, 140, 220))
                                    .underline(),
                            );
                            remaining = &after_close[paren_end + 1..];
                            continue;
                        }
                    }
                }
                render_inline_rest(ui, "[");
                remaining = after_bracket;
                continue;
            }
            // Try *italic* or _italic_
            if let Some((delim, start)) = find_italic_start(remaining) {
                if start > 0 {
                    render_inline_rest(ui, &remaining[..start]);
                }
                let after_open = &remaining[start + delim.len()..];
                if let Some(end) = after_open.find(delim) {
                    ui.label(egui::RichText::new(&after_open[..end]).italics());
                    remaining = &after_open[end + delim.len()..];
                    continue;
                } else {
                    render_inline_rest(ui, remaining);
                    break;
                }
            }
            // No special markup found
            render_inline_rest(ui, remaining);
            break;
        }
    });
}

fn find_italic_start(text: &str) -> Option<(&'static str, usize)> {
    let star_pos = text.find('*').and_then(|i| {
        if text[i..].starts_with("**") { None } else { Some(i) }
    });
    let under_pos = text.find('_');
    match (star_pos, under_pos) {
        (Some(s), Some(u)) => if s <= u { Some(("*", s)) } else { Some(("_", u)) },
        (Some(s), None) => Some(("*", s)),
        (None, Some(u)) => Some(("_", u)),
        (None, None) => None,
    }
}

fn render_inline_rest(ui: &mut egui::Ui, text: &str) {
    if !text.is_empty() {
        ui.label(text);
    }
}

fn load_image_as_texture(ctx: &egui::Context, src: &str) -> Option<egui::TextureHandle> {
    if src.starts_with("http://") || src.starts_with("https://") {
        return None;
    }
    let path = src.strip_prefix("file://").unwrap_or(src);
    // Reject absolute paths and any path traversal components (e.g. "..").
    // Images must be relative paths within the working directory.
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        return None;
    }
    if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: Vec<egui::Color32> = rgba
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    let color_image = egui::ColorImage { size: [w as usize, h as usize], pixels };
    Some(ctx.load_texture(src, color_image, egui::TextureOptions::LINEAR))
}
