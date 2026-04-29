use egui::{Color32, Layout, RichText, Stroke};

use crate::app::AppState;
use crate::theme::colors::{
    BG_ELEVATED, BG_HOVER, BORDER_DEFAULT, STATUS_ERROR, TEXT_PRIMARY, TEXT_SECONDARY,
};

const DANGER_HOVER_BG: Color32 = Color32::from_rgb(42, 26, 26);

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    let devices_info: Vec<(String, String)> = state
        .devices
        .iter()
        .map(|d| {
            let label = d
                .model
                .as_deref()
                .map_or_else(|| d.serial.clone(), |m| format!("{} ({})", m, d.serial));
            (d.serial.clone(), label)
        })
        .collect();

    let has_devices = !devices_info.is_empty();

    let selected_label = state
        .selected_device
        .as_deref()
        .and_then(|serial| devices_info.iter().find(|(s, _)| s == serial))
        .map_or_else(
            || {
                if has_devices {
                    "디바이스 선택...".to_string()
                } else {
                    "디바이스 없음".to_string()
                }
            },
            |(_, label)| label.clone(),
        );

    ui.horizontal_centered(|ui| {
        ui.add_space(8.0);

        ui.add_enabled_ui(has_devices, |ui| {
            egui::ComboBox::from_id_source("toolbar_device_select")
                .width(200.0)
                .selected_text(&selected_label)
                .show_ui(ui, |ui| {
                    let mut new_selection = state.selected_device.clone();
                    for (serial, label) in &devices_info {
                        let is_selected =
                            state.selected_device.as_deref() == Some(serial.as_str());
                        if ui.selectable_label(is_selected, label).clicked() {
                            new_selection = Some(serial.clone());
                        }
                    }
                    if new_selection != state.selected_device && new_selection.is_some() {
                        state.is_streaming = true;
                    }
                    state.selected_device = new_selection;
                });
        });

        ui.add_space(8.0);

        let toggle_label = if state.is_streaming { "⏸ 정지" } else { "▶ 재개" };
        if ghost_button(ui, toggle_label).clicked() {
            state.is_streaming = !state.is_streaming;
        }

        ui.add_space(4.0);

        if danger_button(ui, "🗑 초기화").clicked() {
            if let Ok(mut buf) = state.log_buffer.lock() {
                buf.clear();
            }
            state.filtered_indices.clear();
            state.filter_dirty = true;
        }

        ui.add_space(4.0);

        if ghost_button(ui, "💾 저장").clicked() {
            state.save_requested = true;
        }

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            if ghost_button(ui, "?").clicked() {
                state.show_help = !state.show_help;
            }

            ui.add_space(4.0);

            if ghost_button(ui, "설정").clicked() {
                state.show_settings = !state.show_settings;
            }
        });
    });
}

fn ghost_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let w = &mut ui.style_mut().visuals.widgets;
        w.inactive.weak_bg_fill = Color32::TRANSPARENT;
        w.inactive.bg_fill = Color32::TRANSPARENT;
        w.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
        w.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        w.hovered.weak_bg_fill = BG_HOVER;
        w.hovered.bg_fill = BG_HOVER;
        w.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
        w.hovered.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        w.active.weak_bg_fill = BG_ELEVATED;
        w.active.bg_fill = BG_ELEVATED;
        w.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
        w.active.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        ui.button(text)
    })
    .inner
}

fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let w = &mut ui.style_mut().visuals.widgets;
        w.inactive.weak_bg_fill = Color32::TRANSPARENT;
        w.inactive.bg_fill = Color32::TRANSPARENT;
        w.inactive.fg_stroke = Stroke::new(1.0, STATUS_ERROR);
        w.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        w.hovered.weak_bg_fill = DANGER_HOVER_BG;
        w.hovered.bg_fill = DANGER_HOVER_BG;
        w.hovered.fg_stroke = Stroke::new(1.0, STATUS_ERROR);
        w.hovered.bg_stroke = Stroke::new(1.0, STATUS_ERROR);
        w.active.weak_bg_fill = DANGER_HOVER_BG;
        w.active.bg_fill = DANGER_HOVER_BG;
        w.active.fg_stroke = Stroke::new(1.0, STATUS_ERROR);
        w.active.bg_stroke = Stroke::new(1.0, STATUS_ERROR);
        ui.button(text)
    })
    .inner
}
