use std::collections::{HashMap, HashSet};
use std::io::Write as IoWrite;

#[derive(Clone)]
pub struct ColumnWidths {
    pub time: f32,
    pub level: f32,
    pub tag: f32,
    pub pid: f32,
    pub pkg: f32,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self { time: 160.0, level: 32.0, tag: 140.0, pid: 60.0, pkg: 160.0 }
    }
}
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::adb::{list_devices, query_pid_map, AdbManager};
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
    pub focused_log_id: Option<u64>,
    pub detail_log_id: Option<u64>,
    pub selected_log_ids: HashSet<u64>,
    pub last_click_idx: Option<usize>,
    pub show_settings: bool,
    pub settings_just_opened: bool,
    pub show_help: bool,
    pub help_just_opened: bool,
    pub focus_search: bool,
    pub save_requested: bool,
    pub settings: AppSettings,
    pub device_poll_tx: mpsc::Sender<()>,
    pub adb_error: Option<String>,
    pub search_debounce_until: Option<Instant>,
    pub save_status: Option<(String, Instant)>,
    pub last_error: Option<String>,
    pub last_error_time: Option<Instant>,
    pub pid_map: HashMap<u32, String>,
    pub col_widths: ColumnWidths,
    pub scroll_to_row: Option<usize>,
    pub table_visible_height: f32,
    pub table_top_y: f32,
    pub drag_select_anchor: Option<usize>,
    pub filter_buf_len: usize,
    pub show_package_filter: bool,
    pub package_filter_anchor: egui::Pos2,
    pub user_packages: HashSet<String>,
    pub seen_pids: HashSet<u32>,
    pub active_packages: Vec<String>,
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
    pid_map_rx: mpsc::Receiver<HashMap<u32, String>>,
    pid_map_tx: mpsc::Sender<HashMap<u32, String>>,
    pid_refresh_task: Option<tokio::task::JoinHandle<()>>,
    last_drain: Instant,
    active_serial: Option<String>,
    last_filter_state: crate::model::FilterState,
    user_pkg_tx: mpsc::Sender<HashSet<String>>,
    user_pkg_rx: mpsc::Receiver<HashSet<String>>,
}

impl NlogcatApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = settings::load();

        match settings.theme {
            Theme::Dark => cc.egui_ctx.set_visuals(crate::theme::dark_visuals()),
            Theme::Light => cc.egui_ctx.set_visuals(crate::theme::light_visuals()),
        }
        cc.egui_ctx
            .set_fonts(crate::theme::fonts::build_font_definitions_with_family(&settings.font_family));

        let log_buffer = Arc::new(Mutex::new(LogBuffer::new(settings.max_buffer_lines)));
        let (log_tx, log_rx) = mpsc::channel::<LogEntry>(10_000);
        let (device_poll_tx, mut device_poll_rx) = mpsc::channel::<()>(1);
        let (device_result_tx, device_rx) = mpsc::channel::<Vec<Device>>(4);
        let (pid_map_tx, pid_map_rx) = mpsc::channel::<HashMap<u32, String>>(4);

        tokio::task::spawn_blocking(|| crate::theme::font_scanner::warm_up());

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
            focused_log_id: None,
            detail_log_id: None,
            selected_log_ids: HashSet::new(),
            last_click_idx: None,
            show_settings: false,
            settings_just_opened: false,
            show_help: false,
            help_just_opened: false,
            focus_search: false,
            save_requested: false,
            settings,
            device_poll_tx,
            adb_error: initial_adb_error,
            search_debounce_until: None,
            save_status: None,
            last_error: None,
            last_error_time: None,
            pid_map: HashMap::new(),
            col_widths: ColumnWidths::default(),
            scroll_to_row: None,
            table_visible_height: 400.0,
            table_top_y: 0.0,
            drag_select_anchor: None,
            filter_buf_len: 0,
            show_package_filter: false,
            package_filter_anchor: egui::Pos2::ZERO,
            user_packages: HashSet::new(),
            seen_pids: HashSet::new(),
            active_packages: Vec::new(),
        };

        let (user_pkg_tx, user_pkg_rx) = mpsc::channel::<HashSet<String>>(4);

        Self {
            state,
            device_rx,
            log_tx,
            adb_manager,
            pid_map_rx,
            pid_map_tx,
            pid_refresh_task: None,
            last_drain: Instant::now(),
            active_serial: None,
            last_filter_state: crate::model::FilterState::default(),
            user_pkg_tx,
            user_pkg_rx,
        }
    }
}

impl eframe::App for NlogcatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let had_new_data = self.drain_log_channel();
        self.drain_device_channel();
        self.drain_user_packages();
        self.drain_pid_map_channel();
        self.check_search_debounce();
        self.recompute_filter_if_dirty();
        self.handle_save_request();
        self.check_stream_health();
        self.manage_streaming();
        self.tick_error_dismiss();

        let (copy_requested, size_changed, scroll_delta, focus_search) = ctx.input(|i| {
            let copy = (i.modifiers.ctrl && i.key_pressed(egui::Key::C))
                || i.events.iter().any(|e| matches!(e, egui::Event::Copy));
            let size = i.viewport().inner_rect.map(|r| (r.width(), r.height()));
            let scroll = if i.modifiers.ctrl { i.raw_scroll_delta.y } else { 0.0 };
            let search = i.modifiers.ctrl && i.key_pressed(egui::Key::F);
            (copy, size, scroll, search)
        });
        if focus_search {
            self.state.focus_search = true;
        }
        if let Some((w, h)) = size_changed {
            self.state.settings.window_width = w;
            self.state.settings.window_height = h;
        }
        if scroll_delta != 0.0 {
            let delta = if scroll_delta > 0.0 { 1.0_f32 } else { -1.0_f32 };
            self.state.settings.font_size =
                (self.state.settings.font_size + delta).clamp(8.0, 24.0);
            let _ = settings::save(&self.state.settings);
        }


        if self.state.selected_device.is_some() {
            crate::ui::main_view::render(ctx, &mut self.state);
        } else {
            crate::ui::empty_view::render(ctx, &mut self.state);
        }

        // Apply clipboard copy AFTER all UI renders.
        // If a selectable label already wrote copied_text (e.g. message area),
        // respect that and do not overwrite with log-row content.
        if copy_requested && !self.state.selected_log_ids.is_empty() {
            let text = self.build_copy_content();
            let count = self.state.selected_log_ids.len();
            let mut did_copy = false;
            ctx.output_mut(|o| {
                if o.copied_text.is_empty() {
                    o.copied_text = text;
                    did_copy = true;
                }
            });
            if did_copy {
                self.state.save_status = Some((format!("{count}줄 복사됨"), Instant::now()));
            }
        }

        // Drive repaints: 100ms when data is flowing, 500ms when streaming but quiet.
        if had_new_data {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else if self.state.is_streaming {
            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = settings::save(&self.state.settings);
    }
}

impl NlogcatApp {
    fn drain_log_channel(&mut self) -> bool {
        const MAX_PER_FRAME: usize = 500;
        let mut new_entries = Vec::new();
        for _ in 0..MAX_PER_FRAME {
            match self.state.log_rx.try_recv() {
                Ok(entry) => new_entries.push(entry),
                Err(_) => break,
            }
        }
        if new_entries.is_empty() {
            return false;
        }

        for entry in &new_entries {
            if self.state.seen_pids.insert(entry.pid) {
                let pkg = self.state.pid_map.get(&entry.pid)
                    .filter(|p| !p.is_empty())
                    .cloned();
                if let Some(pkg) = pkg {
                    if !self.state.active_packages.contains(&pkg) {
                        self.state.active_packages.push(pkg);
                    }
                }
            }
        }

        let Ok(mut buffer) = self.state.log_buffer.lock() else {
            return false;
        };
        for entry in new_entries {
            buffer.push(entry);
        }
        drop(buffer);

        let now = Instant::now();
        if now.duration_since(self.last_drain) >= Duration::from_millis(100) {
            self.last_drain = now;
            if self.state.auto_scroll {
                self.state.scroll_to_bottom = true;
            }
            self.state.filter_dirty = true;
        }
        true
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

        let Ok(buffer) = self.state.log_buffer.lock() else {
            return;
        };

        let current_len = buffer.entries().len();
        let filter_unchanged = self.state.filter == self.last_filter_state;
        let only_appended = current_len > self.state.filter_buf_len;

        let safe_to_append = filter_unchanged
            && only_appended
            && self.state.filter_buf_len > 0
            && current_len < buffer.max_size();

        if safe_to_append {
            let q_low = if !self.state.filter.case_sensitive && !self.state.filter.search_query.is_empty() {
                Some(self.state.filter.search_query.to_lowercase())
            } else {
                None
            };
            let start = self.state.filter_buf_len;
            for (abs_idx, entry) in buffer.entries().iter().enumerate().skip(start) {
                if crate::engine::filter::FilterEngine::matches(
                    entry,
                    &self.state.filter,
                    &self.state.pid_map,
                    q_low.as_deref(),
                ) {
                    self.state.filtered_indices.push(abs_idx);
                }
            }
        } else {
            self.state.filtered_indices = crate::engine::filter::FilterEngine::compute_indices(
                &buffer,
                &self.state.filter,
                &self.state.pid_map,
            );
            self.last_filter_state = self.state.filter.clone();
        }

        self.state.filter_buf_len = current_len;
        self.state.filter_dirty = false;
    }

    fn drain_device_channel(&mut self) {
        while let Ok(devices) = self.device_rx.try_recv() {
            self.state.devices = devices;
        }
    }

    fn drain_pid_map_channel(&mut self) {
        let mut updated = false;
        while let Ok(map) = self.pid_map_rx.try_recv() {
            self.state.pid_map = map;
            updated = true;
        }
        if updated {
            let pids: Vec<u32> = self.state.seen_pids.iter().cloned().collect();
            for pid in pids {
                let pkg = self.state.pid_map.get(&pid)
                    .filter(|p| !p.is_empty())
                    .cloned();
                if let Some(pkg) = pkg {
                    if !self.state.active_packages.contains(&pkg) {
                        self.state.active_packages.push(pkg);
                    }
                }
            }
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

        let result: std::io::Result<()> = (|| {
            let file = std::fs::File::create(&path)?;
            let mut writer = std::io::BufWriter::new(file);
            let Ok(buffer) = self.state.log_buffer.lock() else {
                return Err(std::io::Error::other("버퍼 잠금 오류"));
            };
            let entries = buffer.entries();
            for &idx in &self.state.filtered_indices {
                if let Some(entry) = entries.get(idx) {
                    writer.write_all(entry.raw.as_bytes())?;
                    writer.write_all(b"\n")?;
                }
            }
            drop(buffer);
            writer.flush()
        })();

        match result {
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
            let should_stop = self.adb_manager.as_ref().is_some_and(|m| m.is_streaming());
            if should_stop {
                if let Some(ref mut mgr) = self.adb_manager {
                    mgr.stop_stream();
                }
                self.stop_pid_refresh();
            }
            if self.state.is_streaming {
                self.state.is_streaming = false;
            }
            self.active_serial = None;
            return;
        };

        // Collect decisions from the borrow of adb_manager, then act after borrow ends.
        enum Action {
            None,
            StartStream { adb_path: std::path::PathBuf },
            StartFailed(String),
            StopStream,
        }

        let device_changed = self.active_serial.as_deref() != Some(serial.as_str());

        let action = match self.adb_manager {
            None => {
                if self.state.is_streaming {
                    Action::StartFailed("ADB를 찾을 수 없습니다".to_string())
                } else {
                    Action::None
                }
            }
            Some(ref mut mgr) => {
                if self.state.is_streaming && (!mgr.is_streaming() || device_changed) {
                    let adb_path = mgr.adb_path.clone();
                    match mgr.start_stream(&serial, self.log_tx.clone()) {
                        Ok(()) => Action::StartStream { adb_path },
                        Err(e) => Action::StartFailed(e.to_string()),
                    }
                } else if !self.state.is_streaming && mgr.is_streaming() {
                    mgr.stop_stream();
                    Action::StopStream
                } else {
                    Action::None
                }
            }
        };

        match action {
            Action::None => {}
            Action::StartStream { adb_path } => {
                self.active_serial = Some(serial.clone());
                self.state.user_packages.clear();
                self.state.seen_pids.clear();
                self.state.active_packages.clear();
                self.spawn_user_packages_load(adb_path.clone(), serial.clone());
                self.start_pid_refresh(adb_path, serial);
            }
            Action::StartFailed(msg) => {
                self.state.is_streaming = false;
                self.active_serial = None;
                self.state.set_error(msg);
            }
            Action::StopStream => {
                self.active_serial = None;
                self.stop_pid_refresh();
            }
        }
    }

    fn start_pid_refresh(&mut self, adb_path: std::path::PathBuf, serial: String) {
        self.stop_pid_refresh();
        let tx = self.pid_map_tx.clone();
        self.pid_refresh_task = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let path = adb_path.clone();
                let ser = serial.clone();
                let map = tokio::task::spawn_blocking(move || query_pid_map(&path, &ser))
                    .await
                    .unwrap_or_default();
                if tx.send(map).await.is_err() {
                    break;
                }
            }
        }));
    }

    fn stop_pid_refresh(&mut self) {
        if let Some(task) = self.pid_refresh_task.take() {
            task.abort();
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
            self.stop_pid_refresh();
            self.active_serial = None;
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



    fn drain_user_packages(&mut self) {
        while let Ok(pkgs) = self.user_pkg_rx.try_recv() {
            self.state.user_packages = pkgs;
        }
    }

    fn spawn_user_packages_load(&mut self, adb_path: std::path::PathBuf, serial: String) {
        let tx = self.user_pkg_tx.clone();
        tokio::task::spawn_blocking(move || {
            let pkgs = crate::adb::package::fetch_user_packages(&adb_path, &serial);
            let _ = tx.blocking_send(pkgs);
        });
    }


    fn build_copy_content(&self) -> String {
        let Ok(buffer) = self.state.log_buffer.lock() else {
            return String::new();
        };
        let entries = buffer.entries();
        let mut out = String::new();
        for &idx in &self.state.filtered_indices {
            if let Some(entry) = entries.get(idx) {
                if self.state.selected_log_ids.contains(&entry.id) {
                    out.push_str(&entry.raw);
                    out.push('\n');
                }
            }
        }
        out
    }
}
