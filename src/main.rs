#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// 이후 태스크에서 연결될 항목들에 대한 임시 허용
#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_errors_doc)]

mod app;
mod adb;
mod engine;
mod model;
mod ui;
mod config;
mod theme;

fn main() {
    // eframe 진입점 (TASK-09에서 구현)
}
