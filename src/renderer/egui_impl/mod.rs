pub mod components;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::protocol::{A2NInput, A2NOutput, ButtonAction, Component, IpcMessage, OutputStatus};
use crate::renderer::Renderer;

// ── IPC command (used internally between IPC threads and the egui app) ────────

pub(crate) enum IpcCommand {
    Update {
        input: A2NInput,
        response_tx: std::sync::mpsc::SyncSender<A2NOutput>,
    },
    Close,
}

// ── FormState ─────────────────────────────────────────────────────────────────

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
                    state.text_values.insert(id.clone(), default_value.clone().unwrap_or_default());
                }
                Component::Textarea { id, default_value, .. } => {
                    state.text_values.insert(id.clone(), default_value.clone().unwrap_or_default());
                }
                Component::NumberInput { id, default_value, .. } => {
                    state.number_values.insert(id.clone(), default_value.unwrap_or(0.0));
                }
                Component::DatePicker { id, default_value, .. } => {
                    state.text_values.insert(id.clone(), default_value.clone().unwrap_or_default());
                }
                Component::TimePicker { id, default_value, .. } => {
                    state.text_values.insert(id.clone(), default_value.clone().unwrap_or_default());
                }
                Component::Dropdown { id, default_value, .. } => {
                    state.text_values.insert(id.clone(), default_value.clone().unwrap_or_default());
                }
                Component::Checkbox { id, default_value, .. } => {
                    state.bool_values.insert(id.clone(), *default_value);
                }
                Component::CheckboxGroup { id, default_values, .. } => {
                    state.checkbox_group_values.insert(id.clone(), default_values.clone());
                }
                Component::RadioGroup { id, default_value, .. } => {
                    state.text_values.insert(id.clone(), default_value.clone().unwrap_or_default());
                }
                Component::Slider { id, default_value, min, .. } => {
                    state.number_values.insert(id.clone(), default_value.unwrap_or(*min));
                }
                Component::FileUpload { id, .. } => {
                    state.text_values.insert(id.clone(), String::new());
                }
                Component::Password { id, .. } => {
                    state.text_values.insert(id.clone(), String::new());
                }
                Component::Rating { id, default_value, .. } => {
                    state.number_values.insert(id.clone(), default_value.unwrap_or(0) as f64);
                }
                Component::Toggle { id, default_value, .. } => {
                    state.bool_values.insert(id.clone(), *default_value);
                }
                Component::Card { children, .. } | Component::Row { children, .. } => {
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
                | Component::FileUpload { id, .. }
                | Component::Password { id, .. } => {
                    if let Some(v) = self.text_values.get(id) {
                        values.insert(id.clone(), serde_json::Value::String(v.clone()));
                    }
                }
                Component::NumberInput { id, .. }
                | Component::Slider { id, .. }
                | Component::Rating { id, .. } => {
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
                Component::Checkbox { id, .. } | Component::Toggle { id, .. } => {
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
                Component::Card { children, .. } | Component::Row { children, .. } => {
                    self.collect_from_components(values, children);
                }
                _ => {}
            }
        }
    }
}

// ── One-shot renderer ─────────────────────────────────────────────────────────

pub struct EguiRenderer;

impl EguiRenderer {
    pub fn new() -> Self { EguiRenderer }
}

impl Default for EguiRenderer {
    fn default() -> Self { Self::new() }
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
        let _ = eframe::run_native(&title, native_options, Box::new(move |cc| {
            setup_fonts_and_style(&cc.egui_ctx);
            Ok(Box::new(app))
        }));

        let result = output_slot.lock().unwrap().take().unwrap_or_else(|| A2NOutput {
            status: OutputStatus::Cancelled,
            values: HashMap::new(),
        });
        result
    }
}

// ── Session daemon ────────────────────────────────────────────────────────────

/// Start a long-lived window managed through a TCP IPC channel.
/// Optionally also binds an HTTP/SSE server and/or a WebSocket server
/// so external agents can connect without the CLI wrapper.
pub fn run_daemon(uuid: &str, sse_port: Option<u16>, ws_port: Option<u16>) {
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind IPC port");
    let port = listener.local_addr().unwrap().port();
    crate::session::write_port(uuid, port);

    let (ipc_tx, ipc_rx) = mpsc::channel::<IpcCommand>();

    // TCP IPC listener (used by `a2n --session UUID` CLI invocations)
    {
        let tx = ipc_tx.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => {
                        let tx2 = tx.clone();
                        thread::spawn(move || handle_ipc(s, tx2));
                    }
                    Err(_) => break,
                }
            }
        });
    }

    // Optional HTTP/SSE server
    if let Some(port) = sse_port {
        crate::server::start_sse(port, ipc_tx.clone());
    }

    // Optional WebSocket server
    if let Some(port) = ws_port {
        crate::server::start_ws(port, ipc_tx.clone());
    }

    // Block until the first form arrives over IPC.
    let (first_input, first_tx) = loop {
        match ipc_rx.recv() {
            Ok(IpcCommand::Update { input, response_tx }) => break (input, response_tx),
            Ok(IpcCommand::Close) | Err(_) => {
                crate::session::remove_port(uuid);
                return;
            }
        }
    };

    let title = first_input.title.clone().unwrap_or_else(|| "A2N Form".to_string());
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([600.0, 500.0])
            .with_resizable(true),
        ..Default::default()
    };

    let app = A2NApp::new_session(first_input, first_tx, ipc_rx, uuid.to_string());
    let _ = eframe::run_native(&title, native_options, Box::new(move |cc| {
        setup_fonts_and_style(&cc.egui_ctx);
        Ok(Box::new(app))
    }));

    crate::session::remove_port(uuid);
}

fn handle_ipc(stream: std::net::TcpStream, tx: std::sync::mpsc::Sender<IpcCommand>) {
    use std::io::{BufRead, BufReader, Write};

    let stream_out = stream.try_clone();
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() && !line.is_empty() {
        if let Ok(msg) = serde_json::from_str::<IpcMessage>(line.trim()) {
            match msg {
                IpcMessage::Update { input } => {
                    let (resp_tx, resp_rx) = std::sync::mpsc::sync_channel(0);
                    tx.send(IpcCommand::Update { input, response_tx: resp_tx }).ok();
                    if let Ok(output) = resp_rx.recv() {
                        if let Ok(mut s) = stream_out {
                            if let Ok(resp) = serde_json::to_string(&output) {
                                let _ = writeln!(s, "{resp}");
                            }
                        }
                    }
                }
                IpcMessage::Close => {
                    tx.send(IpcCommand::Close).ok();
                }
            }
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

struct SessionState {
    uuid: String,
    ipc_rx: std::sync::mpsc::Receiver<IpcCommand>,
    pending_response: Option<std::sync::mpsc::SyncSender<A2NOutput>>,
    waiting: bool,
}

struct A2NApp {
    input: A2NInput,
    state: FormState,
    output_slot: Arc<Mutex<Option<A2NOutput>>>,
    start_time: Instant,
    has_submit_button: bool,
    session: Option<SessionState>,
    banner_text: &'static str,
}

impl A2NApp {
    fn new(input: A2NInput, output_slot: Arc<Mutex<Option<A2NOutput>>>) -> Self {
        let state = FormState::from_input(&input);
        let has_submit_button = Self::check_has_submit(&input.components);
        let locale = detect_ui_language();
        A2NApp {
            input,
            state,
            output_slot,
            start_time: Instant::now(),
            has_submit_button,
            session: None,
            banner_text: security_banner_text(&locale),
        }
    }

    fn new_session(
        input: A2NInput,
        response_tx: std::sync::mpsc::SyncSender<A2NOutput>,
        ipc_rx: std::sync::mpsc::Receiver<IpcCommand>,
        uuid: String,
    ) -> Self {
        let state = FormState::from_input(&input);
        let has_submit_button = Self::check_has_submit(&input.components);
        let locale = detect_ui_language();
        A2NApp {
            input,
            state,
            output_slot: Arc::new(Mutex::new(None)),
            start_time: Instant::now(),
            has_submit_button,
            session: Some(SessionState {
                uuid,
                ipc_rx,
                pending_response: Some(response_tx),
                waiting: false,
            }),
            banner_text: security_banner_text(&locale),
        }
    }

    fn check_has_submit(components: &[Component]) -> bool {
        for c in components {
            match c {
                Component::Button { action, .. } if *action == ButtonAction::Submit => return true,
                Component::Card { children, .. } | Component::Row { children, .. } => {
                    if Self::check_has_submit(children) { return true; }
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
        let is_submit = matches!(result, FormResult::Submitted);
        let status = match result {
            FormResult::Submitted => OutputStatus::Submitted,
            FormResult::Cancelled => OutputStatus::Cancelled,
            FormResult::Timeout => OutputStatus::Timeout,
        };
        let values = self.state.collect_output(&self.input.components);
        let output = A2NOutput { status, values };

        if let Some(session) = &mut self.session {
            if let Some(tx) = session.pending_response.take() {
                let _ = tx.send(output);
            }
            if is_submit {
                session.waiting = true;
            } else {
                let uuid = session.uuid.clone();
                crate::session::remove_port(&uuid);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else {
            *self.output_slot.lock().unwrap() = Some(output);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for A2NApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);

        // ── Security banner (always visible at the top) ───────────────────────
        // Keep banner in CentralPanel (not TopBottomPanel) so the window width
        // constraint applies and Label::wrap_mode actually wraps the text.
        // It is placed above the ScrollArea so it stays pinned at the top.

        // ── Session mode: poll for IPC commands ──────────────────────────────
        let mut close_daemon = false;
        let mut close_uuid = String::new();

        if let Some(session) = &mut self.session {
            match session.ipc_rx.try_recv() {
                Ok(IpcCommand::Update { input, response_tx }) => {
                    self.input = input;
                    self.state = FormState::from_input(&self.input);
                    self.has_submit_button = Self::check_has_submit(&self.input.components);
                    self.start_time = Instant::now();
                    // Cancel any previous pending client so it doesn't block forever.
                    if let Some(prev_tx) = session.pending_response.take() {
                        let _ = prev_tx.send(A2NOutput {
                            status: OutputStatus::Cancelled,
                            values: HashMap::new(),
                        });
                    }
                    session.pending_response = Some(response_tx);
                    session.waiting = false;
                    let title = self.input.title.clone().unwrap_or_else(|| "A2N Form".to_string());
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
                }
                Ok(IpcCommand::Close) => {
                    if let Some(tx) = session.pending_response.take() {
                        let _ = tx.send(A2NOutput {
                            status: OutputStatus::Cancelled,
                            values: HashMap::new(),
                        });
                    }
                    close_daemon = true;
                    close_uuid = session.uuid.clone();
                }
                Err(_) => {}
            }

            if session.waiting && !close_daemon {
                let uuid_prefix = session.uuid.get(..8).unwrap_or(&session.uuid).to_string();
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("⏳  Waiting for the agent to send the next step…")
                                .size(15.0),
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(format!("Session: {uuid_prefix}…")).small().weak(),
                        );
                    });
                });
                ctx.request_repaint_after(Duration::from_millis(100));
                return;
            }
        }

        if close_daemon {
            crate::session::remove_port(&close_uuid);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // ── Timeout check ─────────────────────────────────────────────────────
        if let Some(timeout_secs) = self.input.timeout {
            if self.start_time.elapsed().as_secs() >= timeout_secs {
                self.finalize(ctx, FormResult::Timeout);
                return;
            }
        }

        // ── Handle form result set by component interactions ──────────────────
        if self.state.result.is_some() {
            let result = self.state.result.take().unwrap();
            self.finalize(ctx, result);
            return;
        }

        // ── Main form panel ───────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            // Banner pinned at top, outside the scroll area
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(255, 200, 0))
                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    let wrap_width = ui.available_width();
                    let job = egui::text::LayoutJob::simple(
                        self.banner_text.to_string(),
                        egui::FontId::proportional(14.0),
                        egui::Color32::from_rgb(40, 25, 0),
                        wrap_width,
                    );
                    ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Wrap));
                });
            ui.add_space(4.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(title) = &self.input.title {
                    ui.label(egui::RichText::new(title).size(20.0).strong());
                    ui.add_space(12.0);
                }
                // Split-borrow: `input.components` (immutable) and `state` (mutable)
                // are separate struct fields, so no clone is needed.
                let num_components = self.input.components.len();
                for i in 0..num_components {
                    components::render_component(ui, &self.input.components[i], &mut self.state);
                    ui.add_space(6.0);
                }
                if !self.has_submit_button {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new("Submit").strong().color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(52, 120, 246))
                        .min_size(egui::vec2(90.0, 32.0)),
                    ).clicked() {
                        self.state.result = Some(FormResult::Submitted);
                    }
                }
            });
        });

        if self.input.timeout.is_some() {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }
}

// ── Font & style setup ────────────────────────────────────────────────────────

/// Load a CJK-capable system font as fallback and apply sensible style defaults.
/// Called once at window creation via eframe's `CreationContext`.
fn setup_fonts_and_style(ctx: &egui::Context) {
    // ── CJK fallback font ─────────────────────────────────────────────────────
    // egui ships without CJK glyphs; load the first available system font so
    // that Chinese / Japanese / Korean text in banners and form content renders.
    let cjk_candidates: &[&str] = &[
        r"C:\Windows\Fonts\msyh.ttc",                              // Windows: Microsoft YaHei
        r"C:\Windows\Fonts\simsun.ttc",                            // Windows: SimSun
        "/System/Library/Fonts/PingFang.ttc",                      // macOS
        "/System/Library/Fonts/STHeiti Light.ttc",                 // macOS (older)
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",  // Linux: Noto
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",          // Linux: WenQuanYi
        "/usr/share/fonts/wqy-microhei/wqy-microhei.ttc",
    ];

    let mut fonts = egui::FontDefinitions::default();
    for path in cjk_candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "cjk".to_owned(),
                egui::FontData::from_owned(bytes),
            );
            // Append after the default font so Latin glyphs still use Hackney/Ubuntu.
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("cjk".to_owned());
            break;
        }
    }
    ctx.set_fonts(fonts);

    // ── Style ─────────────────────────────────────────────────────────────────
    let mut style = (*ctx.style()).clone();

    // More breathing room between items and inside buttons.
    style.spacing.item_spacing    = egui::vec2(8.0, 8.0);
    style.spacing.button_padding  = egui::vec2(14.0, 7.0);
    style.spacing.interact_size.y = 26.0; // taller inputs / buttons

    // Rounded corners everywhere.
    let r = egui::Rounding::same(6.0);
    style.visuals.widgets.noninteractive.rounding = r;
    style.visuals.widgets.inactive.rounding       = r;
    style.visuals.widgets.hovered.rounding        = r;
    style.visuals.widgets.active.rounding         = r;
    style.visuals.widgets.open.rounding           = r;
    style.visuals.window_rounding                 = egui::Rounding::same(10.0);
    style.visuals.menu_rounding                   = egui::Rounding::same(8.0);

    ctx.set_style(style);
}

// ── Locale-aware security banner ──────────────────────────────────────────────

/// Detect the best-fit UI language tag from the running system.
/// Returns a short BCP-47-like tag, e.g. "zh-CN", "ja", "fr".
/// The `A2N_LANG` environment variable overrides auto-detection.
fn detect_ui_language() -> String {
    if let Ok(override_lang) = std::env::var("A2N_LANG") {
        if !override_lang.is_empty() {
            return override_lang.replace('_', "-");
        }
    }
    // sys-locale reads $LANG / $LC_ALL on Unix; Win32 GetUserDefaultLocaleName on Windows.
    sys_locale::get_locale()
        .unwrap_or_else(|| "en".to_string())
        .replace('_', "-") // normalise zh_CN → zh-CN
}

/// Return the security banner text for the given locale string.
/// Matching is done on the language subtag (and optionally region).
pub fn security_banner_text(locale: &str) -> &'static str {
    // Split "zh-CN", "zh_CN", "zh-Hant-TW", etc. into (lang, region)
    let mut parts = locale.splitn(3, ['-', '_']);
    let lang = parts.next().unwrap_or("en").to_ascii_lowercase();
    let sub = parts.next().unwrap_or("").to_ascii_uppercase();

    match lang.as_str() {
        "zh" => {
            // zh-TW / zh-HK / zh-MO / zh-Hant → Traditional Chinese
            if matches!(sub.as_str(), "TW" | "HK" | "MO" | "HANT") {
                "⚠  此介面由 AI 代理程式生成。您輸入的內容將傳送給 AI 代理，且可能被他人看到——請勿輸入敏感資訊。"
            } else {
                "⚠  此界面由 AI 代理生成。您输入的内容将发送给 AI 代理，且可能被他人看到——请勿输入敏感信息。"
            }
        }
        "ja" => "⚠  このインターフェースはAIエージェントが生成しました。入力内容はエージェントに送信され、他の人に見られる可能性があります——機密情報を入力しないでください。",
        "ko" => "⚠  이 인터페이스는 AI 에이전트가 생성했습니다. 입력한 내용은 에이전트에게 전송되며 타인이 볼 수 있습니다——민감한 정보를 입력하지 마세요。",
        "fr" => "⚠  Cette interface a été générée par un agent IA. Vos saisies seront envoyées à l'agent et pourraient être vues par d'autres — ne saisissez pas d'informations sensibles.",
        "de" => "⚠  Diese Oberfläche wurde von einem KI-Agenten generiert. Ihre Eingaben werden an den Agenten gesendet und könnten von anderen eingesehen werden — geben Sie keine vertraulichen Daten ein.",
        "es" => "⚠  Esta interfaz fue generada por un agente de IA. Sus entradas se enviarán al agente y pueden ser vistas por otros — no introduzca información sensible.",
        "pt" => "⚠  Esta interface foi gerada por um agente de IA. Suas entradas serão enviadas ao agente e podem ser vistas por outros — não insira informações sensíveis.",
        "it" => "⚠  Questa interfaccia è stata generata da un agente IA. I tuoi dati verranno inviati all'agente e potrebbero essere visti da altri — non inserire informazioni sensibili.",
        "ru" => "⚠  Этот интерфейс создан агентом ИИ. Введённые вами данные будут отправлены агенту и могут быть видны другим — не вводите конфиденциальные данные.",
        "ar" => "⚠  تم إنشاء هذه الواجهة بواسطة وكيل ذكاء اصطناعي. ستُرسَل مدخلاتك إلى الوكيل وقد يراها الآخرون — لا تُدخل معلومات حساسة.",
        "hi" => "⚠  यह इंटरफ़ेस एक AI एजेंट द्वारा बनाया गया है। आपका इनपुट एजेंट को भेजा जाएगा और दूसरों द्वारा देखा जा सकता है — संवेदनशील जानकारी न डालें।",
        "tr" => "⚠  Bu arayüz bir yapay zeka ajanı tarafından oluşturuldu. Girişleriniz ajana gönderilecek ve başkaları tarafından görülebilir — hassas bilgi girmeyin.",
        "nl" => "⚠  Deze interface is gegenereerd door een AI-agent. Uw invoer wordt naar de agent verzonden en kan door anderen worden gezien — voer geen gevoelige informatie in.",
        "pl" => "⚠  Ten interfejs został wygenerowany przez agenta AI. Twoje dane zostaną wysłane do agenta i mogą być widoczne dla innych — nie wprowadzaj poufnych danych.",
        "vi" => "⚠  Giao diện này được tạo bởi tác nhân AI. Thông tin bạn nhập sẽ được gửi đến tác nhân và có thể bị người khác xem — không nhập thông tin nhạy cảm.",
        "th" => "⚠  อินเทอร์เฟซนี้สร้างโดย AI agent ข้อมูลที่คุณป้อนจะถูกส่งไปยัง agent และอาจถูกผู้อื่นเห็น — อย่าป้อนข้อมูลที่ละเอียดอ่อน",
        _ => "⚠  This interface was generated by an AI agent. Your input will be sent to the agent and may be seen by others — do not enter sensitive information.",
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{A2NInput, Component, SelectOption};

    fn make_input(components: Vec<Component>) -> A2NInput {
        A2NInput { title: None, timeout: None, theme: None, components }
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
        let input = make_input(vec![Component::TextField {
            id: "name".to_string(),
            label: None,
            placeholder: None,
            required: false,
            default_value: Some("Alice".to_string()),
        }]);
        let state = FormState::from_input(&input);
        assert_eq!(state.text_values.get("name").map(|s| s.as_str()), Some("Alice"));
    }

    #[test]
    fn test_form_state_init_number_defaults() {
        let input = make_input(vec![Component::NumberInput {
            id: "age".to_string(),
            label: None,
            min: None,
            max: None,
            step: None,
            default_value: Some(25.0),
        }]);
        let state = FormState::from_input(&input);
        assert_eq!(state.number_values.get("age"), Some(&25.0));
    }

    #[test]
    fn test_form_state_init_checkbox() {
        let input = make_input(vec![Component::Checkbox {
            id: "agree".to_string(),
            label: None,
            default_value: true,
        }]);
        let state = FormState::from_input(&input);
        assert_eq!(state.bool_values.get("agree"), Some(&true));
    }

    #[test]
    fn test_form_state_init_checkbox_group() {
        let input = make_input(vec![Component::CheckboxGroup {
            id: "tags".to_string(),
            label: None,
            options: vec![
                SelectOption { value: "a".to_string(), label: "A".to_string() },
                SelectOption { value: "b".to_string(), label: "B".to_string() },
            ],
            default_values: vec!["a".to_string()],
        }]);
        let state = FormState::from_input(&input);
        assert_eq!(state.checkbox_group_values.get("tags"), Some(&vec!["a".to_string()]));
    }

    #[test]
    fn test_form_state_init_slider_uses_min_as_default() {
        let input = make_input(vec![Component::Slider {
            id: "vol".to_string(),
            label: None,
            min: 10.0,
            max: 50.0,
            step: None,
            default_value: None,
        }]);
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
            Component::Checkbox { id: "ok".to_string(), label: None, default_value: false },
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
        let components = vec![Component::Button {
            id: "s".to_string(),
            label: "Submit".to_string(),
            action: ButtonAction::Submit,
        }];
        let input = make_input(components);
        assert!(A2NApp::check_has_submit(&input.components));
    }

    #[test]
    fn test_check_has_submit_false() {
        let components = vec![Component::Text { id: "t".to_string(), content: "hello".to_string() }];
        let input = make_input(components);
        assert!(!A2NApp::check_has_submit(&input.components));
    }

    #[test]
    fn test_security_banner_english() {
        let text = security_banner_text("en-US");
        assert!(text.contains("AI agent"));
    }

    #[test]
    fn test_security_banner_zh_cn() {
        let text = security_banner_text("zh-CN");
        assert!(text.contains("敏感信息"));
    }

    #[test]
    fn test_security_banner_zh_tw() {
        let text = security_banner_text("zh-TW");
        assert!(text.contains("敏感資訊")); // Traditional
    }

    #[test]
    fn test_security_banner_ja() {
        let text = security_banner_text("ja-JP");
        assert!(text.contains("機密情報"));
    }

    #[test]
    fn test_security_banner_de() {
        let text = security_banner_text("de-DE");
        assert!(text.contains("KI-Agenten"));
    }

    #[test]
    fn test_security_banner_unknown_falls_back_to_en() {
        let text = security_banner_text("xx-UNKNOWN");
        assert!(text.contains("AI agent"));
    }

    #[test]
    fn test_check_has_submit_in_card() {
        let components = vec![Component::Card {
            id: "c".to_string(),
            title: None,
            children: vec![Component::Button {
                id: "b".to_string(),
                label: "Go".to_string(),
                action: ButtonAction::Submit,
            }],
        }];
        let input = make_input(components);
        assert!(A2NApp::check_has_submit(&input.components));
    }
}
