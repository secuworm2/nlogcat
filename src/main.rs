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
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime 생성 실패");
    let _guard = rt.enter();

    let settings = crate::config::settings::load();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([settings.window_width, settings.window_height])
            .with_title("nlogcat"),
        ..Default::default()
    };

    eframe::run_native(
        "nlogcat",
        native_options,
        Box::new(|cc| Ok(Box::new(NlogcatApp::new(cc)))),
    )
    .expect("eframe 실행 실패");
}
