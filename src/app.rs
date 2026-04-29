use std::io::Write as IoWrite;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::adb::{list_devices, AdbManager};
use crate::config::settings::{self, AppSettings, Theme};
use crate::model::{Device, FilterState, LogBuffer, LogEntry};

#[allow(clippy::struct_excessive_bools)]
pub struct AppState {
    pub devices: Vec<Device>,
    pub selected_device: Option<String>,
    pub is_streaming: bool,
    pub log_buffer: Arc<Mutex<LogBuffer>>,
    pub log_rx: mpsc::Receiver<LogEntry>,
    pub filter: FilterState,
    pub filtered_indices: Vec<usize>,
    pub filter_dirty: bool,
    pub auto_scroll: bool,
    pub scroll_to_bottom: bool,
    pub selected_log_id: Option<u64>,
    pub show_settings: bool,
    pub show_help: bool,
    pub save_requested: bool,
    pub settings: AppSettings,
    pub device_poll_tx: mpsc::Sender<()>,
    pub adb_error: Option<String>,
    pub search_debounce_until: Option<Instant>,
    pub save_status: Option<(String, Instant)>,
    pub last_error: Option<String>,
    pub last_error_time: Option<Instant>,
}

impl AppState {
    pub fn set_error(&mut self, msg: String) {
        self.last_error = Some(msg);
        self.last_error_time = Some(Instant::now());
    }
}

pub struct NlogcatApp {
    pub state: AppState,
    device_rx: mpsc::Receiver<Vec<Device>>,
    pub log_tx: mpsc::Sender<LogEntry>,
    adb_manager: Option<AdbManager>,
}

impl NlogcatApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = settings::load();

        match settings.theme {
            Theme::Dark => cc.egui_ctx.set_visuals(crate::theme::dark_visuals()),
            Theme::Light => cc.egui_ctx.set_visuals(crate::theme::light_visuals()),
        }
        cc.egui_ctx
            .set_fonts(crate::theme::fonts::build_font_definitions());

        let log_buffer = Arc::new(Mutex::new(LogBuffer::new(settings.max_buffer_lines)));
        let (log_tx, log_rx) = mpsc::channel::<LogEntry>(10_000);
        let (device_poll_tx, mut device_poll_rx) = mpsc::channel::<()>(1);
        let (device_result_tx, device_rx) = mpsc::channel::<Vec<Device>>(4);

        let adb_path_result = AdbManager::resolve_adb_path(settings.adb_path.as_deref());
        let initial_adb_error = adb_path_result.as_ref().err().map(ToString::to_string);
        let adb_path = adb_path_result.ok();
        let adb_manager = adb_path
            .as_ref()
            .map(|p| AdbManager::new(p.clone()));

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            loop {
                tokio::select! {
                    _ = interval.tick() => {},
                    result = device_poll_rx.recv() => {
                        if result.is_none() { break; }
                    }
                }
                if let Some(ref path) = adb_path {
                    let path = path.clone();
                    let devices = match tokio::task::spawn_blocking(move || list_devices(&path)).await {
                        Ok(Ok(d)) => d,
                        Ok(Err(_)) | Err(_) => Vec::new(),
                    };
                    if device_result_tx.send(devices).await.is_err() {
                        break;
                    }
                }
            }
        });

        let state = AppState {
            devices: Vec::new(),
            selected_device: None,
            is_streaming: false,
            log_buffer,
            log_rx,
            filter: FilterState::default(),
            filtered_indices: Vec::new(),
            filter_dirty: true,
            auto_scroll: settings.auto_scroll,
            scroll_to_bottom: false,
            selected_log_id: None,
            show_settings: false,
            show_help: false,
            save_requested: false,
            settings,
            device_poll_tx,
            adb_error: initial_adb_error,
            search_debounce_until: None,
            save_status: None,
            last_error: None,
            last_error_time: None,
        };

        Self {
            state,
            device_rx,
            log_tx,
            adb_manager,
        }
    }
}

impl eframe::App for NlogcatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_log_channel();
        self.drain_device_channel();
        self.check_search_debounce();
        self.recompute_filter_if_dirty();
        self.handle_save_request();
        self.check_stream_health();
        self.manage_streaming();
        self.tick_error_dismiss();

        ctx.input(|i| {
            if let Some(rect) = i.viewport().inner_rect {
                self.state.settings.window_width = rect.width();
                self.state.settings.window_height = rect.height();
            }
        });

        if self.state.selected_device.is_some() {
            crate::ui::main_view::render(ctx, &mut self.state);
        } else {
            crate::ui::empty_view::render(ctx, &mut self.state);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = settings::save(&self.state.settings);
    }
}

impl NlogcatApp {
    fn drain_log_channel(&mut self) {
        const MAX_PER_FRAME: usize = 500;
        let mut new_entries = Vec::new();
        for _ in 0..MAX_PER_FRAME {
            match self.state.log_rx.try_recv() {
                Ok(entry) => new_entries.push(entry),
                Err(_) => break,
            }
        }
        if new_entries.is_empty() {
            return;
        }

        if self.state.auto_scroll {
            self.state.scroll_to_bottom = true;
        }

        let Ok(mut buffer) = self.state.log_buffer.lock() else {
            return;
        };
        for entry in new_entries {
            buffer.push(entry);
        }
        self.state.filter_dirty = true;
    }

    fn check_search_debounce(&mut self) {
        if let Some(until) = self.state.search_debounce_until {
            if Instant::now() >= until {
                self.state.filter_dirty = true;
                self.state.search_debounce_until = None;
            }
        }
    }

    fn recompute_filter_if_dirty(&mut self) {
        if !self.state.filter_dirty {
            return;
        }
        let new_indices = {
            let Ok(buffer) = self.state.log_buffer.lock() else {
                return;
            };
            crate::engine::filter::FilterEngine::compute_indices(&buffer, &self.state.filter)
        };
        self.state.filtered_indices = new_indices;
        self.state.filter_dirty = false;
    }

    fn drain_device_channel(&mut self) {
        while let Ok(devices) = self.device_rx.try_recv() {
            self.state.devices = devices;
        }
    }

    fn handle_save_request(&mut self) {
        if !self.state.save_requested {
            return;
        }
        self.state.save_requested = false;

        let path = rfd::FileDialog::new()
            .add_filter("Log", &["txt", "log"])
            .save_file();

        let Some(path) = path else {
            return;
        };

        let content = {
            let Ok(buffer) = self.state.log_buffer.lock() else {
                self.state.set_error("저장 실패: 버퍼 잠금 오류".to_string());
                return;
            };
            let mut out = String::with_capacity(buffer.len() * 80);
            for entry in buffer.entries() {
                out.push_str(&entry.raw);
                out.push('\n');
            }
            out
        };

        match std::fs::File::create(&path).and_then(|mut f| f.write_all(content.as_bytes())) {
            Ok(()) => {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("파일")
                    .to_string();
                self.state.save_status = Some((format!("저장 완료: {filename}"), Instant::now()));
            }
            Err(e) => {
                self.state.set_error(format!("저장 실패: {e}"));
            }
        }
    }

    fn manage_streaming(&mut self) {
        let Some(serial) = self.state.selected_device.clone() else {
            if let Some(ref mut mgr) = self.adb_manager {
                if mgr.is_streaming() {
                    mgr.stop_stream();
                }
            }
            if self.state.is_streaming {
                self.state.is_streaming = false;
            }
            return;
        };

        match self.adb_manager {
            None => {
                if self.state.is_streaming {
                    self.state.is_streaming = false;
                    self.state
                        .set_error("ADB를 찾을 수 없습니다".to_string());
                }
            }
            Some(ref mut mgr) => {
                if self.state.is_streaming && !mgr.is_streaming() {
                    match mgr.start_stream(&serial, self.log_tx.clone()) {
                        Ok(()) => {}
                        Err(e) => {
                            self.state.is_streaming = false;
                            self.state.set_error(e.to_string());
                        }
                    }
                } else if !self.state.is_streaming && mgr.is_streaming() {
                    mgr.stop_stream();
                }
            }
        }
    }

    fn check_stream_health(&mut self) {
        if !self.state.is_streaming {
            return;
        }
        let finished = self
            .adb_manager
            .as_ref()
            .is_some_and(AdbManager::task_finished);
        if finished {
            if let Some(ref mut mgr) = self.adb_manager {
                mgr.stop_stream();
            }
            self.state.is_streaming = false;
            self.state
                .set_error("스트림이 예기치 않게 종료되었습니다".to_string());
        }
    }

    fn tick_error_dismiss(&mut self) {
        if let Some(t) = self.state.last_error_time {
            if t.elapsed() >= Duration::from_secs(3) {
                self.state.last_error = None;
                self.state.last_error_time = None;
            }
        }
    }
}
