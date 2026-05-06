use egui::{Frame, Margin, RichText, Rounding, Stroke};

use crate::app::AppState;
use crate::config::settings::{self, AppSettings, Theme};
use crate::theme::colors::TEXT_PRIMARY;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_settings {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.show_settings = false;
        return;
    }

    let window_fill = ctx.style().visuals.window_fill;
    let border_color = ctx.style().visuals.window_stroke.color;

    let mut close = false;
    let mut changed = false;

    let use_center = state.settings_just_opened;
    state.settings_just_opened = false;

    let screen_rect = ctx.screen_rect();
    let default_pos = screen_rect.center() - egui::vec2(216.0, 190.0);

    let mut window = egui::Window::new("__settings_panel")
        .title_bar(false)
        .resizable(false)
        .default_pos(default_pos)
        .min_width(400.0)
        .frame(
            Frame::none()
                .fill(window_fill)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(16.0))
                .stroke(Stroke::new(1.0, border_color)),
        );

    if use_center {
        window = window.current_pos(default_pos);
    }

    window.show(ctx, |ui| {
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
    let text_color = ui.visuals().text_color();
    ui.horizontal(|ui| {
        ui.label(RichText::new("설정").strong().color(text_color));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("X").clicked() {
                *close = true;
            }
        });
    });
}

fn render_display_section(ui: &mut egui::Ui, state: &mut AppState, changed: &mut bool) {
    let text_color = ui.visuals().text_color();
    let weak_text = ui.visuals().weak_text_color();

    ui.label(RichText::new("표시").strong().color(text_color));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("폰트 크기").color(weak_text));
        ui.add_space(8.0);
        ui.label(RichText::new(format!("{:.0}", state.settings.font_size)).color(text_color));
        if ui.small_button("−").clicked() {
            state.settings.font_size = (state.settings.font_size - 1.0).clamp(8.0, 24.0);
            *changed = true;
        }
        if ui.small_button("+").clicked() {
            state.settings.font_size = (state.settings.font_size + 1.0).clamp(8.0, 24.0);
            *changed = true;
        }
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("폰트").color(weak_text));
        ui.add_space(8.0);
        let prev_font = state.settings.font_family.clone();
        egui::ComboBox::from_id_source("font_family_combo")
            .selected_text(&state.settings.font_family)
            .width(200.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.settings.font_family,
                    "Default".to_string(),
                    "기본",
                );
                for info in crate::theme::font_scanner::system_monospace_fonts() {
                    ui.selectable_value(
                        &mut state.settings.font_family,
                        info.display_name.clone(),
                        &info.display_name,
                    );
                }
            });
        if state.settings.font_family != prev_font {
            let new_fonts = crate::theme::fonts::build_font_definitions_with_family(
                &state.settings.font_family,
            );
            ui.ctx().set_fonts(new_fonts);
            *changed = true;
        }
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("테마").color(weak_text));
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
    let text_color = ui.visuals().text_color();
    let weak_text = ui.visuals().weak_text_color();

    ui.label(RichText::new("성능").strong().color(text_color));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("최대 버퍼 줄 수").color(weak_text));
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
    let text_color = ui.visuals().text_color();
    let weak_text = ui.visuals().weak_text_color();

    ui.label(RichText::new("ADB").strong().color(text_color));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("adb 경로").color(weak_text));
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
