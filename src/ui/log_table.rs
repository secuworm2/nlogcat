use egui::{Color32, FontId};

use crate::app::AppState;
use crate::model::LogEntry;
use crate::theme::colors::{
    level_label_color, level_row_bg, BG_SELECTED, BG_SURFACE, TEXT_PRIMARY, TEXT_SECONDARY,
};

// Column widths per TRD §3.2
const COL_TIME: f32 = 160.0;
const COL_LV: f32 = 32.0;
const COL_TAG: f32 = 140.0;
const COL_PID: f32 = 60.0;
const ROW_HEIGHT: f32 = 20.0;
const HEADER_HEIGHT: f32 = 24.0;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    render_header(ui);

    let mut clicked_id: Option<u64> = None;

    {
        let filtered_indices = &state.filtered_indices;
        let log_buffer = &state.log_buffer;
        let selected_log_id = state.selected_log_id;
        let auto_scroll = state.auto_scroll;

        let total_rows = {
            let buf = log_buffer.lock().unwrap();
            if filtered_indices.is_empty() {
                buf.len()
            } else {
                filtered_indices.len()
            }
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(auto_scroll)
            .show_rows(ui, ROW_HEIGHT, total_rows, |ui, row_range| {
                let buf = log_buffer.lock().unwrap();
                let entries = buf.entries();

                for row_idx in row_range {
                    let entry_idx = if filtered_indices.is_empty() {
                        row_idx
                    } else {
                        filtered_indices[row_idx]
                    };

                    if let Some(entry) = entries.get(entry_idx) {
                        let is_selected = selected_log_id == Some(entry.id);
                        let entry_id = entry.id;
                        if render_row(ui, entry, is_selected).clicked() {
                            clicked_id = Some(entry_id);
                        }
                    }
                }
            });
    }

    if let Some(id) = clicked_id {
        state.selected_log_id = Some(id);
    }
}

fn render_header(ui: &mut egui::Ui) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, HEADER_HEIGHT), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, BG_SURFACE);

    let painter = ui.painter();
    let font = FontId::proportional(11.0);
    let x = rect.min.x + 4.0;
    let y = rect.center().y;

    for (col_x, label) in header_cols(x) {
        painter.text(
            egui::pos2(col_x, y),
            egui::Align2::LEFT_CENTER,
            label,
            font.clone(),
            TEXT_SECONDARY,
        );
    }
}

fn header_cols(x: f32) -> [(f32, &'static str); 5] {
    [
        (x, "시간"),
        (x + COL_TIME, "Lv"),
        (x + COL_TIME + COL_LV, "태그"),
        (x + COL_TIME + COL_LV + COL_TAG, "PID"),
        (x + COL_TIME + COL_LV + COL_TAG + COL_PID, "메시지"),
    ]
}

fn render_row(ui: &mut egui::Ui, entry: &LogEntry, is_selected: bool) -> egui::Response {
    let w = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(w, ROW_HEIGHT), egui::Sense::click());

    let base_bg = level_row_bg(entry.level);
    let bg = if is_selected {
        BG_SELECTED
    } else if response.hovered() {
        brighten(base_bg)
    } else {
        base_bg
    };
    ui.painter().rect_filled(rect, 0.0, bg);

    let painter = ui.painter();
    let font = FontId::monospace(12.0);
    let x = rect.min.x + 4.0;
    let y = rect.center().y;

    // Time
    painter.text(
        egui::pos2(x, y),
        egui::Align2::LEFT_CENTER,
        format!("{} {}", entry.date, entry.time),
        font.clone(),
        TEXT_SECONDARY,
    );

    // Level badge
    painter.text(
        egui::pos2(x + COL_TIME, y),
        egui::Align2::LEFT_CENTER,
        entry.level.label(),
        font.clone(),
        level_label_color(entry.level),
    );

    // Tag
    painter.text(
        egui::pos2(x + COL_TIME + COL_LV, y),
        egui::Align2::LEFT_CENTER,
        &entry.tag,
        font.clone(),
        TEXT_PRIMARY,
    );

    // PID
    painter.text(
        egui::pos2(x + COL_TIME + COL_LV + COL_TAG, y),
        egui::Align2::LEFT_CENTER,
        entry.pid.to_string(),
        font.clone(),
        TEXT_SECONDARY,
    );

    // Message
    painter.text(
        egui::pos2(x + COL_TIME + COL_LV + COL_TAG + COL_PID, y),
        egui::Align2::LEFT_CENTER,
        &entry.message,
        font,
        TEXT_PRIMARY,
    );

    response
}

fn brighten(color: Color32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    Color32::from_rgba_unmultiplied(
        r.saturating_add(20),
        g.saturating_add(20),
        b.saturating_add(20),
        a,
    )
}
