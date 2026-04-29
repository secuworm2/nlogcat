use egui::{Color32, Layout, Stroke};

use crate::app::AppState;
use crate::theme::colors::STATUS_ERROR;

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
        ui.separator();
        ui.add_space(8.0);

        let toggle_label = if state.is_streaming { "정지" } else { "재개" };
        if flat_button(ui, toggle_label).clicked() {
            state.is_streaming = !state.is_streaming;
        }

        ui.add_space(2.0);

        if danger_button(ui, "초기화").clicked() {
            if let Ok(mut buf) = state.log_buffer.lock() {
                buf.clear();
            }
            state.filtered_indices.clear();
            state.filter_dirty = true;
        }

        ui.add_space(2.0);

        if flat_button(ui, "저장").clicked() {
            state.save_requested = true;
        }

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            if flat_button(ui, "?").clicked() {
                state.show_help = !state.show_help;
            }

            ui.add_space(2.0);

            if flat_button(ui, "설정").clicked() {
                state.show_settings = !state.show_settings;
            }
        });
    });
}

/// Borderless flat button — no border in any state, subtle bg on hover.
fn flat_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let text_color = ui.visuals().text_color();
        let weak_color = ui.visuals().weak_text_color();
        let hover_bg = ui.visuals().widgets.hovered.bg_fill;
        let active_bg = ui.visuals().widgets.active.bg_fill;

        let w = &mut ui.style_mut().visuals.widgets;
        w.inactive.weak_bg_fill = Color32::TRANSPARENT;
        w.inactive.bg_fill = Color32::TRANSPARENT;
        w.inactive.fg_stroke = Stroke::new(1.0, weak_color);
        w.inactive.bg_stroke = Stroke::NONE;
        w.hovered.weak_bg_fill = hover_bg;
        w.hovered.bg_fill = hover_bg;
        w.hovered.fg_stroke = Stroke::new(1.0, text_color);
        w.hovered.bg_stroke = Stroke::NONE;
        w.active.weak_bg_fill = active_bg;
        w.active.bg_fill = active_bg;
        w.active.fg_stroke = Stroke::new(1.0, text_color);
        w.active.bg_stroke = Stroke::NONE;
        ui.button(text)
    })
    .inner
}

fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let dark_mode = ui.visuals().dark_mode;
        let hover_bg = if dark_mode {
            Color32::from_rgb(42, 26, 26)
        } else {
            Color32::from_rgba_unmultiplied(220, 38, 38, 30)
        };

        let w = &mut ui.style_mut().visuals.widgets;
        w.inactive.weak_bg_fill = Color32::TRANSPARENT;
        w.inactive.bg_fill = Color32::TRANSPARENT;
        w.inactive.fg_stroke = Stroke::new(1.0, STATUS_ERROR);
        w.inactive.bg_stroke = Stroke::NONE;
        w.hovered.weak_bg_fill = hover_bg;
        w.hovered.bg_fill = hover_bg;
        w.hovered.fg_stroke = Stroke::new(1.0, STATUS_ERROR);
        w.hovered.bg_stroke = Stroke::NONE;
        w.active.weak_bg_fill = hover_bg;
        w.active.bg_fill = hover_bg;
        w.active.fg_stroke = Stroke::new(1.0, STATUS_ERROR);
        w.active.bg_stroke = Stroke::NONE;
        ui.button(text)
    })
    .inner
}
