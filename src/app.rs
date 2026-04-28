use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::adb::{list_devices, AdbManager};
use crate::config::settings::{self, AppSettings, Theme};
use crate::model::{Device, FilterState, LogBuffer, LogEntry};

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
    pub selected_log_id: Option<u64>,
    pub show_settings: bool,
    pub show_help: bool,
    pub save_requested: bool,
    pub settings: AppSettings,
    pub device_poll_tx: mpsc::Sender<()>,
    pub adb_error: Option<String>,
}

pub struct NlogcatApp {
    pub state: AppState,
    device_rx: mpsc::Receiver<Vec<Device>>,
    // AdbStreamer에 전달하기 위해 보관
    pub log_tx: mpsc::Sender<LogEntry>,
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

        let adb_path = AdbManager::resolve_adb_path(settings.adb_path.as_deref()).ok();

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
                    let devices = tokio::task::spawn_blocking(move || {
                        list_devices(&path).unwrap_or_default()
                    })
                    .await
                    .unwrap_or_default();
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
            filter_dirty: false,
            auto_scroll: settings.auto_scroll,
            selected_log_id: None,
            show_settings: false,
            show_help: false,
            save_requested: false,
            settings,
            device_poll_tx,
            adb_error: None,
        };

        Self {
            state,
            device_rx,
            log_tx,
        }
    }
}

impl eframe::App for NlogcatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_log_channel();
        self.drain_device_channel();

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
        let mut entries = Vec::new();
        for _ in 0..MAX_PER_FRAME {
            match self.state.log_rx.try_recv() {
                Ok(entry) => entries.push(entry),
                Err(_) => break,
            }
        }
        if !entries.is_empty() {
            let mut buffer = self.state.log_buffer.lock().unwrap();
            for entry in entries {
                buffer.push(entry);
            }
            self.state.filter_dirty = true;
        }
    }

    fn drain_device_channel(&mut self) {
        while let Ok(devices) = self.device_rx.try_recv() {
            self.state.devices = devices;
        }
    }
}
