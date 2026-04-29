use std::collections::HashMap;

use egui::{Color32, FontId};

use crate::app::AppState;
use crate::engine::search::SearchEngine;
use crate::model::LogEntry;
use crate::theme::colors::{
    level_label_color, level_row_bg, BG_SELECTED, BG_SURFACE, TEXT_PRIMARY, TEXT_SECONDARY,
};

// Column widths per TRD §3.2
const COL_TIME: f32 = 160.0;
const COL_LV: f32 = 32.0;
const COL_TAG: f32 = 140.0;
const COL_PID: f32 = 60.0;
const COL_PKG: f32 = 160.0;
const HEADER_HEIGHT: f32 = 24.0;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    let font_size = state.settings.font_size;
    let row_height = font_size + 8.0;

    render_header(ui);

    let search_query = state.filter.search_query.clone();
    let case_sensitive = state.filter.case_sensitive;

    let mut clicked_id: Option<u64> = None;
    let should_scroll_to_bottom = state.scroll_to_bottom;

    // Borrow scope: immutable borrows on state.filtered_indices and state.log_buffer.
    // Returns scroll metrics so we can update state after the borrows are released.
    let (scroll_offset_y, content_height, visible_height) = {
        let filtered_indices = &state.filtered_indices;
        let log_buffer = &state.log_buffer;
        let selected_log_id = state.selected_log_id;
        let pid_map = &state.pid_map;

        let total_rows = filtered_indices.len();

        let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]);

        if should_scroll_to_bottom {
            // f32::MAX causes egui layout assertion failure; use a large finite value
            // that always exceeds content height so egui clamps it to the true bottom.
            scroll_area = scroll_area.vertical_scroll_offset(total_rows as f32 * row_height * 2.0);
        }

        let output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
            let Ok(buf) = log_buffer.lock() else { return; };
            let entries = buf.entries();

            for row_idx in row_range {
                let Some(&entry_idx) = filtered_indices.get(row_idx) else {
                    continue;
                };

                if let Some(entry) = entries.get(entry_idx) {
                    let is_selected = selected_log_id == Some(entry.id);
                    let entry_id = entry.id;
                    let pkg_name = pid_map.get(&entry.pid).map(String::as_str).unwrap_or("");
                    if render_row(ui, entry, is_selected, &search_query, case_sensitive, font_size, row_height, pkg_name)
                        .clicked()
                    {
                        clicked_id = Some(entry_id);
                    }
                }
            }
        });

        (output.state.offset.y, output.content_size.y, output.inner_rect.height())
    };

    // Clear the one-shot flag after use
    if should_scroll_to_bottom {
        state.scroll_to_bottom = false;
    }

    // Detect scroll position: at bottom → enable auto_scroll, scrolled up → disable
    if content_height > 0.0 {
        let max_scroll = (content_height - visible_height).max(0.0);
        let at_bottom = scroll_offset_y >= max_scroll - 2.0;

        if at_bottom && !state.auto_scroll {
            state.auto_scroll = true;
        } else if !at_bottom && state.auto_scroll {
            state.auto_scroll = false;
        }
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

fn header_cols(x: f32) -> [(f32, &'static str); 6] {
    [
        (x, "시간"),
        (x + COL_TIME, "Lv"),
        (x + COL_TIME + COL_LV, "태그"),
        (x + COL_TIME + COL_LV + COL_TAG, "PID"),
        (x + COL_TIME + COL_LV + COL_TAG + COL_PID, "패키지"),
        (x + COL_TIME + COL_LV + COL_TAG + COL_PID + COL_PKG, "메시지"),
    ]
}

fn render_row(
    ui: &mut egui::Ui,
    entry: &LogEntry,
    is_selected: bool,
    search_query: &str,
    case_sensitive: bool,
    font_size: f32,
    row_height: f32,
    pkg_name: &str,
) -> egui::Response {
    let w = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(w, row_height), egui::Sense::click());

    let base_bg = level_row_bg(entry.level);
    let bg = if is_selected {
        BG_SELECTED
    } else if response.hovered() {
        brighten(base_bg)
    } else {
        base_bg
    };
    ui.painter().rect_filled(rect, 0.0, bg);

    let font = FontId::monospace(font_size);
    let x = rect.min.x + 4.0;
    let y = rect.center().y;

    // Time (no highlighting)
    ui.painter().text(
        egui::pos2(x, y),
        egui::Align2::LEFT_CENTER,
        format!("{} {}", entry.date, entry.time),
        font.clone(),
        TEXT_SECONDARY,
    );

    // Level badge (no highlighting)
    ui.painter().text(
        egui::pos2(x + COL_TIME, y),
        egui::Align2::LEFT_CENTER,
        entry.level.label(),
        font.clone(),
        level_label_color(entry.level),
    );

    // Tag — highlight if search query matches
    paint_cell(
        ui,
        &entry.tag,
        TEXT_PRIMARY,
        egui::pos2(x + COL_TIME + COL_LV, y),
        font.clone(),
        search_query,
        case_sensitive,
    );

    // PID (no highlighting)
    ui.painter().text(
        egui::pos2(x + COL_TIME + COL_LV + COL_TAG, y),
        egui::Align2::LEFT_CENTER,
        entry.pid.to_string(),
        font.clone(),
        TEXT_SECONDARY,
    );

    // Package name (no highlighting)
    ui.painter().text(
        egui::pos2(x + COL_TIME + COL_LV + COL_TAG + COL_PID, y),
        egui::Align2::LEFT_CENTER,
        pkg_name,
        font.clone(),
        TEXT_SECONDARY,
    );

    // Message — highlight if search query matches
    paint_cell(
        ui,
        &entry.message,
        TEXT_PRIMARY,
        egui::pos2(x + COL_TIME + COL_LV + COL_TAG + COL_PID + COL_PKG, y),
        font,
        search_query,
        case_sensitive,
    );

    response
}

/// Paints a text cell, using a highlighted `LayoutJob` when the search query
/// has matches, and plain `painter.text()` otherwise.
fn paint_cell(
    ui: &mut egui::Ui,
    text: &str,
    base_color: Color32,
    pos: egui::Pos2,
    font: FontId,
    search_query: &str,
    case_sensitive: bool,
) {
    if !search_query.is_empty() {
        let ranges = SearchEngine::highlight_ranges(text, search_query, case_sensitive);
        if !ranges.is_empty() {
            let job = SearchEngine::build_layout_job(text, &ranges, base_color, font);
            let galley = ui.fonts(|f| f.layout_job(job));
            let top_left = egui::pos2(pos.x, pos.y - galley.size().y / 2.0);
            ui.painter().galley(top_left, galley, base_color);
            return;
        }
    }
    ui.painter()
        .text(pos, egui::Align2::LEFT_CENTER, text, font, base_color);
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
