use egui::{Frame, Margin, RichText, Rounding, Stroke};

use crate::app::AppState;
use crate::model::LogEntry;
use crate::theme::colors::{
    level_label_color, BG_ELEVATED, BORDER_DEFAULT, TEXT_PRIMARY, TEXT_SECONDARY,
};

const COPY_FEEDBACK_ID: &str = "detail_copy_time";

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    let Some(log_id) = state.detail_log_id else {
        return;
    };

    // Key handling: ESC closes, ↑/↓ navigate filtered entries
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.detail_log_id = None;
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        navigate(state, -1);
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        navigate(state, 1);
    }

    let entry = {
        let Ok(buffer) = state.log_buffer.lock() else {
            state.detail_log_id = None;
            return;
        };
        buffer.find_by_id(log_id).cloned()
    };

    let Some(entry) = entry else {
        state.detail_log_id = None;
        return;
    };

    let mut close = false;

    egui::Window::new("__detail_modal")
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
            render_content(ui, &entry, &mut close);
        });

    if close {
        state.detail_log_id = None;
    }
}

/// Move `detail_log_id` by `delta` steps within `filtered_indices`.
fn navigate(state: &mut AppState, delta: i64) {
    let Some(log_id) = state.detail_log_id else {
        return;
    };

    // Resolve log_id → buffer index
    let cur_buf_idx = {
        let Ok(buf) = state.log_buffer.lock() else {
            return;
        };
        buf.entries()
            .iter()
            .enumerate()
            .find(|(_, e)| e.id == log_id)
            .map(|(i, _)| i)
    };
    let Some(cur_buf_idx) = cur_buf_idx else {
        return;
    };

    // Find current position in filtered_indices
    let Some(cur_pos) = state
        .filtered_indices
        .iter()
        .position(|&idx| idx == cur_buf_idx)
    else {
        return;
    };

    let len = state.filtered_indices.len();
    let new_pos = (cur_pos as i64 + delta).clamp(0, len as i64 - 1) as usize;
    if new_pos == cur_pos {
        return;
    }

    // Resolve new buffer index → log id
    let new_id = {
        let Ok(buf) = state.log_buffer.lock() else {
            return;
        };
        state
            .filtered_indices
            .get(new_pos)
            .and_then(|&ni| buf.entries().get(ni))
            .map(|e| e.id)
    };

    if let Some(id) = new_id {
        state.detail_log_id = Some(id);
    }
}

fn render_content(ui: &mut egui::Ui, entry: &LogEntry, close: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("로그 상세").strong().color(TEXT_PRIMARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("X").clicked() {
                *close = true;
            }
        });
    });

    ui.separator();
    ui.add_space(8.0);

    egui::Grid::new("detail_fields")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            field_row(ui, "시간", format!("{} {}", entry.date, entry.time), TEXT_PRIMARY);
            ui.end_row();

            ui.label(RichText::new("레벨").color(TEXT_SECONDARY));
            ui.label(
                RichText::new(entry.level.label())
                    .color(level_label_color(entry.level))
                    .monospace(),
            );
            ui.end_row();

            field_row(ui, "태그", entry.tag.clone(), TEXT_PRIMARY);
            ui.end_row();

            field_row(ui, "PID", entry.pid.to_string(), TEXT_PRIMARY);
            ui.end_row();

            field_row(ui, "TID", entry.tid.to_string(), TEXT_PRIMARY);
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    ui.label(RichText::new("메시지").color(TEXT_SECONDARY));
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .max_height(180.0)
        .id_source("detail_msg_scroll")
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(&entry.message).monospace(),
                )
                .selectable(true),
            );
        });

    ui.add_space(8.0);

    let copy_id = egui::Id::new(COPY_FEEDBACK_ID);
    let copy_time: Option<f64> = ui.ctx().data(|d| d.get_temp(copy_id));
    let now = ui.ctx().input(|i| i.time);
    let is_copied = copy_time.is_some_and(|t| now - t < 1.0);

    if is_copied {
        ui.ctx().request_repaint();
    }

    let btn_label = if is_copied { "복사됨!" } else { "메시지 복사" };

    if ui.button(btn_label).clicked() && !is_copied {
        let msg = entry.message.clone();
        ui.output_mut(|o| o.copied_text = msg);
        ui.ctx().data_mut(|d| d.insert_temp(copy_id, now));
    }
}

fn field_row(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    ui.label(RichText::new(label).color(TEXT_SECONDARY));
    ui.label(RichText::new(value).color(color));
}
