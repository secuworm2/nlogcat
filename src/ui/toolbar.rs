use egui::{Color32, Layout, Rect, Sense};

use crate::app::AppState;
use crate::theme::colors::STATUS_ERROR;

const ICON_SIZE: f32 = 13.0;
const BTN_PAD_X: f32 = 8.0;
const BTN_PAD_Y: f32 = 4.0;
const ICON_GAP: f32 = 5.0;
const MIN_BTN_H: f32 = 26.0;

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

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        if state.is_streaming {
            if icon_button(ui, crate::ui::icons::pause, "정지").clicked() {
                state.is_streaming = false;
            }
        } else {
            if icon_button(ui, crate::ui::icons::play, "재개").clicked() {
                state.is_streaming = true;
            }
        }

        ui.add_space(2.0);

        if danger_icon_button(ui, crate::ui::icons::refresh, "초기화").clicked() {
            if let Ok(mut buf) = state.log_buffer.lock() {
                buf.clear();
            }
            state.filtered_indices.clear();
            state.filter_dirty = true;
        }

        ui.add_space(2.0);

        if icon_button(ui, crate::ui::icons::save, "저장").clicked() {
            state.save_requested = true;
        }

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            if flat_text_button(ui, "?").clicked() {
                state.show_help = !state.show_help;
            }

            ui.add_space(2.0);

            if icon_button(ui, crate::ui::icons::gear, "설정").clicked() {
                state.show_settings = !state.show_settings;
            }
        });
    });
}

fn icon_button(
    ui: &mut egui::Ui,
    paint_fn: fn(&egui::Painter, Rect, Color32),
    label: &str,
) -> egui::Response {
    icon_button_impl(ui, paint_fn, label, false)
}

fn danger_icon_button(
    ui: &mut egui::Ui,
    paint_fn: fn(&egui::Painter, Rect, Color32),
    label: &str,
) -> egui::Response {
    icon_button_impl(ui, paint_fn, label, true)
}

fn icon_button_impl(
    ui: &mut egui::Ui,
    paint_fn: fn(&egui::Painter, Rect, Color32),
    label: &str,
    danger: bool,
) -> egui::Response {
    let font_id = egui::FontId::proportional(12.0);

    let text_size = ui.fonts(|f| {
        f.layout_no_wrap(label.to_owned(), font_id.clone(), Color32::PLACEHOLDER)
            .size()
    });

    let content_w = ICON_SIZE + ICON_GAP + text_size.x;
    let content_h = ICON_SIZE.max(text_size.y);
    let btn_size = egui::vec2(
        content_w + 2.0 * BTN_PAD_X,
        (content_h + 2.0 * BTN_PAD_Y).max(MIN_BTN_H),
    );

    let (rect, response) =
        ui.allocate_exact_size(btn_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let normal_fg = ui.visuals().text_color();
        let weak_fg = ui.visuals().weak_text_color();
        let hover_bg = ui.visuals().widgets.hovered.bg_fill;
        let active_bg = ui.visuals().widgets.active.bg_fill;

        let base_fg = if danger { STATUS_ERROR } else { normal_fg };
        let base_weak = if danger {
            Color32::from_rgba_unmultiplied(
                STATUS_ERROR.r(),
                STATUS_ERROR.g(),
                STATUS_ERROR.b(),
                160,
            )
        } else {
            weak_fg
        };

        let (bg, fg) = if response.is_pointer_button_down_on() {
            (active_bg, base_fg)
        } else if response.hovered() {
            (hover_bg, base_fg)
        } else {
            (Color32::TRANSPARENT, base_weak)
        };

        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, 4.0, bg);
        }

        let icon_rect = Rect::from_min_size(
            egui::pos2(rect.min.x + BTN_PAD_X, rect.center().y - ICON_SIZE / 2.0),
            egui::vec2(ICON_SIZE, ICON_SIZE),
        );
        paint_fn(ui.painter(), icon_rect, fg);

        ui.painter().text(
            egui::pos2(icon_rect.max.x + ICON_GAP, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            font_id,
            fg,
        );
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Icon-less flat text button for small controls like "?".
fn flat_text_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let font_id = egui::FontId::proportional(12.0);
    let text_size = ui.fonts(|f| {
        f.layout_no_wrap(label.to_owned(), font_id.clone(), Color32::PLACEHOLDER)
            .size()
    });
    let btn_size = egui::vec2(
        text_size.x + 2.0 * BTN_PAD_X,
        (text_size.y + 2.0 * BTN_PAD_Y).max(MIN_BTN_H),
    );
    let (rect, response) = ui.allocate_exact_size(btn_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let hover_bg = ui.visuals().widgets.hovered.bg_fill;
        let active_bg = ui.visuals().widgets.active.bg_fill;
        let fg = if response.is_pointer_button_down_on() {
            let _ = active_bg;
            ui.painter().rect_filled(rect, 4.0, active_bg);
            ui.visuals().text_color()
        } else if response.hovered() {
            ui.painter().rect_filled(rect, 4.0, hover_bg);
            ui.visuals().text_color()
        } else {
            ui.visuals().weak_text_color()
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font_id,
            fg,
        );
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
