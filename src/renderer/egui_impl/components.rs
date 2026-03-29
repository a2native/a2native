use eframe::egui;

use crate::protocol::{ButtonAction, Component};
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
