use egui::{Color32, FontId, Sense};

use crate::app::AppState;
use crate::theme::colors::{
    BG_HOVER, BG_SURFACE, STATUS_CONNECTED, STATUS_DISCONNECTED, TEXT_PRIMARY, TEXT_SECONDARY,
};

const HEIGHT: f32 = 24.0;
const FONT_SIZE: f32 = 11.0;
const BTN_WIDTH: f32 = 116.0;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    let w = ui.available_width();
    let (bar_rect, _) = ui.allocate_exact_size(egui::vec2(w, HEIGHT), Sense::hover());
    ui.painter().rect_filled(bar_rect, 0.0, BG_SURFACE);

    let font = FontId::proportional(FONT_SIZE);
    let y = bar_rect.center().y;

    // Left: total / filtered counts
    let total = state.log_buffer.lock().unwrap().len();
    let filtered = if state.filtered_indices.is_empty() {
        total
    } else {
        state.filtered_indices.len()
    };
    ui.painter().text(
        egui::pos2(bar_rect.min.x + 8.0, y),
        egui::Align2::LEFT_CENTER,
        format!("전체 {}줄 | 필터 후 {}줄", total, filtered),
        font.clone(),
        TEXT_SECONDARY,
    );

    // Center: connection status
    let (status_text, status_color) = connection_indicator(state);
    ui.painter().text(
        egui::pos2(bar_rect.center().x, y),
        egui::Align2::CENTER_CENTER,
        status_text,
        font.clone(),
        status_color,
    );

    // Right: auto_scroll toggle (interactive area)
    let btn_rect = egui::Rect::from_min_size(
        egui::pos2(bar_rect.max.x - BTN_WIDTH - 8.0, bar_rect.min.y),
        egui::vec2(BTN_WIDTH, HEIGHT),
    );
    let btn_id = ui.id().with("auto_scroll_toggle");
    let btn_resp = ui.interact(btn_rect, btn_id, Sense::click());

    if btn_resp.hovered() {
        ui.painter().rect_filled(btn_rect, 2.0, BG_HOVER);
    }

    let toggle_label = if state.auto_scroll {
        "▼ 하단 고정: ON"
    } else {
        "▼ 하단 고정: OFF"
    };
    ui.painter().text(
        egui::pos2(btn_rect.max.x - 4.0, y),
        egui::Align2::RIGHT_CENTER,
        toggle_label,
        font,
        TEXT_PRIMARY,
    );

    if btn_resp.clicked() {
        state.auto_scroll = !state.auto_scroll;
    }
}

fn connection_indicator(state: &AppState) -> (String, Color32) {
    if let Some(serial) = &state.selected_device {
        if let Some(device) = state.devices.iter().find(|d| &d.serial == serial) {
            let model = device.model.as_deref().unwrap_or("Unknown");
            return (
                format!("● 연결됨: {} ({})", model, serial),
                STATUS_CONNECTED,
            );
        }
    }
    ("○ 연결 끊김".to_owned(), STATUS_DISCONNECTED)
}
