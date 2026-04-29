use crate::app::{AppState, ColumnWidths};
use crate::engine::search::SearchEngine;
use crate::model::LogEntry;
use crate::theme::colors::{level_label_color, level_row_bg};
use egui::{Color32, FontId};

const HEADER_HEIGHT: f32 = 24.0;
const HANDLE_W: f32 = 8.0;
const MIN_COL_W: f32 = 30.0;

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    let font_size = state.settings.font_size;
    let row_height = font_size + 8.0;

    render_header(ui, &mut state.col_widths);

    let search_query = state.filter.search_query.clone();
    let case_sensitive = state.filter.case_sensitive;

    // Arrow key navigation, Ctrl+A, Delete (only when detail modal is not open)
    if state.detail_log_id.is_none() {
        let (up, down, ctrl_a, delete) = ui.ctx().input(|i| (
            i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::ArrowDown),
            i.modifiers.ctrl && i.key_pressed(egui::Key::A),
            i.key_pressed(egui::Key::Delete),
        ));
        if up { navigate_focus(state, -1); }
        if down { navigate_focus(state, 1); }
        if ctrl_a {
            if let Ok(buf) = state.log_buffer.lock() {
                let entries = buf.entries();
                for &idx in &state.filtered_indices {
                    if let Some(e) = entries.get(idx) {
                        state.selected_log_ids.insert(e.id);
                    }
                }
            }
        }
        if delete && !state.selected_log_ids.is_empty() {
            let focused_deleted = state.focused_log_id
                .is_some_and(|id| state.selected_log_ids.contains(&id));
            if let Ok(mut buf) = state.log_buffer.lock() {
                buf.remove_by_ids(&state.selected_log_ids);
            }
            state.selected_log_ids.clear();
            state.filtered_indices.clear();
            state.filter_dirty = true;
            state.last_click_idx = None;
            if focused_deleted { state.focused_log_id = None; }
        }
    }

    let mut single_clicked: Option<(usize, u64)> = None;
    let mut double_clicked_id: Option<u64> = None;
    let should_scroll_to_bottom = state.scroll_to_bottom;
    let scroll_to_row = state.scroll_to_row;
    let table_visible_height = state.table_visible_height;
    let row_height_full = row_height + ui.spacing().item_spacing.y;

    let modifiers = ui.ctx().input(|i| i.modifiers);
    let dark_mode = ui.visuals().dark_mode;

    let (scroll_offset_y, content_height, visible_height) = {
        let filtered_indices = &state.filtered_indices;
        let log_buffer = &state.log_buffer;
        let selected_log_ids = &state.selected_log_ids;
        let detail_log_id = state.detail_log_id;
        let focused_log_id = state.focused_log_id;
        let pid_map = &state.pid_map;
        let widths = &state.col_widths;

        let total_rows = filtered_indices.len();

        let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]).drag_to_scroll(false);
        if should_scroll_to_bottom {
            scroll_area =
                scroll_area.vertical_scroll_offset(total_rows as f32 * row_height * 2.0);
        } else if let Some(target_row) = scroll_to_row {
            let row_y = target_row as f32 * row_height_full;
            let target_offset = (row_y - table_visible_height / 2.0 + row_height_full / 2.0).max(0.0);
            scroll_area = scroll_area.vertical_scroll_offset(target_offset);
        }

        let output = scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
            let Ok(buf) = log_buffer.lock() else {
                return;
            };
            let entries = buf.entries();

            for row_idx in row_range {
                let Some(&entry_idx) = filtered_indices.get(row_idx) else {
                    continue;
                };

                if let Some(entry) = entries.get(entry_idx) {
                    let is_selected = selected_log_ids.contains(&entry.id)
                        || detail_log_id == Some(entry.id)
                        || focused_log_id == Some(entry.id);
                    let entry_id = entry.id;
                    let pkg_name = pid_map.get(&entry.pid).map(String::as_str).unwrap_or("");
                    let resp = render_row(
                        ui,
                        entry,
                        is_selected,
                        &search_query,
                        case_sensitive,
                        font_size,
                        row_height,
                        pkg_name,
                        widths,
                        dark_mode,
                    );
                    if resp.double_clicked() {
                        double_clicked_id = Some(entry_id);
                    } else if resp.clicked() {
                        single_clicked = Some((row_idx, entry_id));
                    }
                }
            }
        });

        (output.state.offset.y, output.content_size.y, output.inner_rect.height())
    };

    if should_scroll_to_bottom {
        state.scroll_to_bottom = false;
    }
    if scroll_to_row.is_some() {
        state.scroll_to_row = None;
    }
    state.table_visible_height = visible_height;

    if content_height > 0.0 {
        let max_scroll = (content_height - visible_height).max(0.0);
        let at_bottom = scroll_offset_y >= max_scroll - 2.0;
        if at_bottom && !state.auto_scroll {
            state.auto_scroll = true;
        } else if !at_bottom && state.auto_scroll {
            state.auto_scroll = false;
        }
    }

    if let Some(id) = double_clicked_id {
        state.detail_log_id = Some(id);
    }

    if let Some((row_idx, entry_id)) = single_clicked {
        state.focused_log_id = Some(entry_id);
        if modifiers.ctrl {
            if state.selected_log_ids.contains(&entry_id) {
                state.selected_log_ids.remove(&entry_id);
            } else {
                state.selected_log_ids.insert(entry_id);
            }
            state.last_click_idx = Some(row_idx);
        } else if modifiers.shift {
            if let Some(anchor) = state.last_click_idx {
                let lo = anchor.min(row_idx);
                let hi = anchor.max(row_idx);
                let Ok(buf) = state.log_buffer.lock() else {
                    return;
                };
                let entries = buf.entries();
                for pos in lo..=hi {
                    if let Some(&idx) = state.filtered_indices.get(pos) {
                        if let Some(e) = entries.get(idx) {
                            state.selected_log_ids.insert(e.id);
                        }
                    }
                }
            } else {
                state.selected_log_ids.clear();
                state.selected_log_ids.insert(entry_id);
                state.last_click_idx = Some(row_idx);
            }
        } else {
            state.selected_log_ids.clear();
            state.selected_log_ids.insert(entry_id);
            state.last_click_idx = Some(row_idx);
        }
    }
}

fn navigate_focus(state: &mut AppState, delta: i64) {
    state.auto_scroll = false;
    state.scroll_to_bottom = false;

    let cur_pos = state.focused_log_id.and_then(|id| {
        let Ok(buf) = state.log_buffer.lock() else { return None; };
        let cur_buf_idx = buf.entries().iter().enumerate()
            .find(|(_, e)| e.id == id)
            .map(|(i, _)| i)?;
        state.filtered_indices.iter().position(|&idx| idx == cur_buf_idx)
    });

    let len = state.filtered_indices.len();
    if len == 0 { return; }

    let new_pos = match cur_pos {
        Some(pos) => (pos as i64 + delta).clamp(0, len as i64 - 1) as usize,
        None if delta > 0 => 0,
        None => len - 1,
    };

    if cur_pos == Some(new_pos) { return; }

    let new_id = {
        let Ok(buf) = state.log_buffer.lock() else { return; };
        state.filtered_indices.get(new_pos)
            .and_then(|&ni| buf.entries().get(ni))
            .map(|e| e.id)
    };

    if let Some(id) = new_id {
        state.focused_log_id = Some(id);
        state.selected_log_ids.clear();
        state.selected_log_ids.insert(id);
        state.last_click_idx = Some(new_pos);
        state.scroll_to_row = Some(new_pos);
    }
}

fn render_header(ui: &mut egui::Ui, widths: &mut ColumnWidths) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, HEADER_HEIGHT), egui::Sense::hover());
    let header_bg = ui.visuals().faint_bg_color;
    let text_color = ui.visuals().weak_text_color();
    ui.painter().rect_filled(rect, 0.0, header_bg);

    let x = rect.min.x + 4.0;
    let y = rect.center().y;
    let font = FontId::proportional(11.0);

    let lv_x = x + widths.time;
    let tag_x = lv_x + widths.level;
    let pid_x = tag_x + widths.tag;
    let pkg_x = pid_x + widths.pid;
    let msg_x = pkg_x + widths.pkg;

    for (col_x, label) in [
        (x, "시간"),
        (lv_x, "Lv"),
        (tag_x, "태그"),
        (pid_x, "PID"),
        (pkg_x, "패키지"),
        (msg_x, "메시지"),
    ] {
        ui.painter().text(
            egui::pos2(col_x, y),
            egui::Align2::LEFT_CENTER,
            label,
            font.clone(),
            text_color,
        );
    }

    let boundaries = [lv_x, tag_x, pid_x, pkg_x, msg_x];
    let handle_ids = ["cr_time", "cr_lv", "cr_tag", "cr_pid", "cr_pkg"];
    let mut drag: Option<(usize, f32)> = None;

    for (i, (&bx, &hid)) in boundaries.iter().zip(handle_ids.iter()).enumerate() {
        let hr = egui::Rect::from_center_size(
            egui::pos2(bx, rect.center().y),
            egui::vec2(HANDLE_W, HEADER_HEIGHT),
        );
        let resp = ui.interact(hr, ui.id().with(hid), egui::Sense::drag());

        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            ui.painter().vline(
                bx,
                rect.min.y..=rect.max.y,
                egui::Stroke::new(1.0, Color32::from_white_alpha(80)),
            );
        }

        if resp.dragged() {
            drag = Some((i, resp.drag_delta().x));
        }
    }

    if let Some((i, dx)) = drag {
        match i {
            0 => widths.time = (widths.time + dx).max(MIN_COL_W),
            1 => widths.level = (widths.level + dx).max(20.0),
            2 => widths.tag = (widths.tag + dx).max(MIN_COL_W),
            3 => widths.pid = (widths.pid + dx).max(MIN_COL_W),
            4 => widths.pkg = (widths.pkg + dx).max(MIN_COL_W),
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_row(
    ui: &mut egui::Ui,
    entry: &LogEntry,
    is_selected: bool,
    search_query: &str,
    case_sensitive: bool,
    font_size: f32,
    row_height: f32,
    pkg_name: &str,
    widths: &ColumnWidths,
    dark_mode: bool,
) -> egui::Response {
    let w = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(w, row_height), egui::Sense::click());

    let selection_bg = ui.visuals().selection.bg_fill;
    let text_color = ui.visuals().text_color();
    let weak_text = ui.visuals().weak_text_color();

    let base_bg = level_row_bg(entry.level, dark_mode);
    let bg = if is_selected {
        selection_bg
    } else if response.hovered() {
        if dark_mode {
            brighten(base_bg)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 12)
        }
    } else {
        base_bg
    };
    ui.painter().rect_filled(rect, 0.0, bg);

    let font = FontId::monospace(font_size);
    let x = rect.min.x + 4.0;
    let y = rect.center().y;

    let lv_x = x + widths.time;
    let tag_x = lv_x + widths.level;
    let pid_x = tag_x + widths.tag;
    let pkg_x = pid_x + widths.pid;
    let msg_x = pkg_x + widths.pkg;

    let col_clip = |start: f32, width: f32| {
        egui::Rect::from_min_max(
            egui::pos2(start, rect.min.y),
            egui::pos2(start + width - 4.0, rect.max.y),
        )
    };

    ui.painter()
        .with_clip_rect(col_clip(x, widths.time))
        .text(
            egui::pos2(x, y),
            egui::Align2::LEFT_CENTER,
            format!("{} {}", entry.date, entry.time),
            font.clone(),
            weak_text,
        );

    ui.painter()
        .with_clip_rect(col_clip(lv_x, widths.level))
        .text(
            egui::pos2(lv_x, y),
            egui::Align2::LEFT_CENTER,
            entry.level.label(),
            font.clone(),
            level_label_color(entry.level, dark_mode),
        );

    paint_cell(
        ui,
        &entry.tag,
        text_color,
        egui::pos2(tag_x, y),
        font.clone(),
        search_query,
        case_sensitive,
        col_clip(tag_x, widths.tag),
    );

    ui.painter()
        .with_clip_rect(col_clip(pid_x, widths.pid))
        .text(
            egui::pos2(pid_x, y),
            egui::Align2::LEFT_CENTER,
            entry.pid.to_string(),
            font.clone(),
            weak_text,
        );

    ui.painter()
        .with_clip_rect(col_clip(pkg_x, widths.pkg))
        .text(
            egui::pos2(pkg_x, y),
            egui::Align2::LEFT_CENTER,
            pkg_name,
            font.clone(),
            weak_text,
        );

    let msg_clip = egui::Rect::from_min_max(
        egui::pos2(msg_x, rect.min.y),
        egui::pos2(rect.max.x, rect.max.y),
    );
    paint_cell(
        ui,
        &entry.message,
        text_color,
        egui::pos2(msg_x, y),
        font,
        search_query,
        case_sensitive,
        msg_clip,
    );

    response
}

fn paint_cell(
    ui: &mut egui::Ui,
    text: &str,
    base_color: Color32,
    pos: egui::Pos2,
    font: FontId,
    search_query: &str,
    case_sensitive: bool,
    clip_rect: egui::Rect,
) {
    if !search_query.is_empty() {
        let ranges = SearchEngine::highlight_ranges(text, search_query, case_sensitive);
        if !ranges.is_empty() {
            let job = SearchEngine::build_layout_job(text, &ranges, base_color, font);
            let galley = ui.fonts(|f| f.layout_job(job));
            let top_left = egui::pos2(pos.x, pos.y - galley.size().y / 2.0);
            ui.painter().with_clip_rect(clip_rect).galley(top_left, galley, base_color);
            return;
        }
    }
    ui.painter()
        .with_clip_rect(clip_rect)
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
