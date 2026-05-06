#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_errors_doc)]

mod app;
mod adb;
mod engine;
mod model;
mod ui;
mod config;
mod theme;

use app::NlogcatApp;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{info}");
        let _ = std::fs::write("nlogcat_crash.log", msg);
    }));

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("tokio runtime 생성 실패: {e}");
            std::process::exit(1);
        }
    };
    let _guard = rt.enter();

    let settings = crate::config::settings::load();

    let icon = egui::IconData {
        rgba: include_bytes!("../assets/icon32_rgba.bin").to_vec(),
        width: 32,
        height: 32,
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([settings.window_width, settings.window_height])
            .with_title("nlogcat")
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "nlogcat",
        native_options,
        Box::new(|cc| Ok(Box::new(NlogcatApp::new(cc)))),
    ) {
        eprintln!("eframe 실행 실패: {e}");
    }
}
