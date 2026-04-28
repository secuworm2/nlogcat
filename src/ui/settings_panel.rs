use egui::{Frame, Margin, RichText, Rounding, Stroke};

use crate::app::AppState;
use crate::config::settings::{self, AppSettings, Theme};
use crate::theme::colors::{BG_ELEVATED, BORDER_DEFAULT, OVERLAY_SCRIM, TEXT_PRIMARY, TEXT_SECONDARY};

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_settings {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.show_settings = false;
        return;
    }

    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("settings_scrim"),
    ))
    .rect_filled(ctx.screen_rect(), 0.0, OVERLAY_SCRIM);

    let mut close = false;
    let mut changed = false;

    egui::Window::new("__settings_panel")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(400.0)
        .frame(
            Frame::none()
                .fill(BG_ELEVATED)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(16.0))
                .stroke(Stroke::new(1.0, BORDER_DEFAULT)),
        )
        .show(ctx, |ui| {
            render_header(ui, &mut close);
            ui.separator();
            ui.add_space(8.0);
            render_display_section(ui, state, &mut changed);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            render_performance_section(ui, state, &mut changed);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            render_adb_section(ui, state, &mut changed);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            render_reset_button(ui, state, &mut changed);
        });

    if changed {
        let _ = settings::save(&state.settings);
    }

    if close {
        state.show_settings = false;
    }
}

fn render_header(ui: &mut egui::Ui, close: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("설정").strong().color(TEXT_PRIMARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("✕").clicked() {
                *close = true;
            }
        });
    });
}

fn render_display_section(ui: &mut egui::Ui, state: &mut AppState, changed: &mut bool) {
    ui.label(RichText::new("표시").strong().color(TEXT_PRIMARY));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("폰트 크기").color(TEXT_SECONDARY));
        ui.add_space(8.0);
        ui.label(RichText::new(format!("{:.0}", state.settings.font_size)).color(TEXT_PRIMARY));
        if ui.small_button("−").clicked() {
            state.settings.font_size = (state.settings.font_size - 1.0).clamp(10.0, 18.0);
            *changed = true;
        }
        if ui.small_button("+").clicked() {
            state.settings.font_size = (state.settings.font_size + 1.0).clamp(10.0, 18.0);
            *changed = true;
        }
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("테마").color(TEXT_SECONDARY));
        ui.add_space(8.0);
        let prev_theme = state.settings.theme.clone();
        ui.radio_value(&mut state.settings.theme, Theme::Dark, "다크");
        ui.radio_value(&mut state.settings.theme, Theme::Light, "라이트");
        if state.settings.theme != prev_theme {
            apply_theme(ui.ctx(), &state.settings.theme);
            *changed = true;
        }
    });
}

fn render_performance_section(ui: &mut egui::Ui, state: &mut AppState, changed: &mut bool) {
    ui.label(RichText::new("성능").strong().color(TEXT_PRIMARY));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("최대 버퍼 줄 수").color(TEXT_SECONDARY));
        ui.add_space(8.0);
        let resp = ui.add(
            egui::DragValue::new(&mut state.settings.max_buffer_lines)
                .range(1_000_usize..=500_000_usize)
                .speed(100.0),
        );
        if resp.changed() {
            *changed = true;
        }
    });
}

fn render_adb_section(ui: &mut egui::Ui, state: &mut AppState, changed: &mut bool) {
    ui.label(RichText::new("ADB").strong().color(TEXT_PRIMARY));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("adb 경로").color(TEXT_SECONDARY));
        ui.add_space(8.0);

        let mut path_str = state.settings.adb_path.clone().unwrap_or_default();
        let resp = ui.add(
            egui::TextEdit::singleline(&mut path_str)
                .hint_text("자동 감지")
                .desired_width(200.0),
        );
        if resp.changed() {
            state.settings.adb_path = if path_str.is_empty() {
                None
            } else {
                Some(path_str)
            };
            *changed = true;
        }

        if ui.button("찾기").clicked() {
            // std::process로 Windows Explorer를 열어 ADB 경로 탐색을 돕는다
            let _ = std::process::Command::new("explorer.exe")
                .arg(".")
                .spawn();
        }
    });
}

fn render_reset_button(ui: &mut egui::Ui, state: &mut AppState, changed: &mut bool) {
    if ui.button("기본값으로 초기화").clicked() {
        let window_width = state.settings.window_width;
        let window_height = state.settings.window_height;
        state.settings = AppSettings::default();
        state.settings.window_width = window_width;
        state.settings.window_height = window_height;
        apply_theme(ui.ctx(), &state.settings.theme);
        *changed = true;
    }
}

fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    match theme {
        Theme::Dark => ctx.set_visuals(crate::theme::dark_visuals()),
        Theme::Light => ctx.set_visuals(crate::theme::light_visuals()),
    }
}
