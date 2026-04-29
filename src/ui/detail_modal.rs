use egui::{Frame, Margin, RichText, Rounding, Stroke};

use crate::app::AppState;
use crate::model::LogEntry;
use crate::theme::colors::level_label_color;


pub fn render(ctx: &egui::Context, state: &mut AppState) {
    let Some(log_id) = state.detail_log_id else {
        return;
    };

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

    let window_fill = ctx.style().visuals.window_fill;
    let border_color = ctx.style().visuals.window_stroke.color;

    let mut close = false;

    egui::Window::new("__detail_modal")
        .title_bar(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .min_width(400.0)
        .frame(
            Frame::none()
                .fill(window_fill)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(16.0))
                .stroke(Stroke::new(1.0, border_color)),
        )
        .show(ctx, |ui| {
            render_content(ui, &entry, &mut close);
        });

    if close {
        state.detail_log_id = None;
    }
}

fn navigate(state: &mut AppState, delta: i64) {
    let Some(log_id) = state.detail_log_id else {
        return;
    };

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
    let dark_mode = ui.visuals().dark_mode;
    let text_color = ui.visuals().text_color();
    let weak_text = ui.visuals().weak_text_color();

    ui.horizontal(|ui| {
        ui.label(RichText::new("로그 상세").strong().color(text_color));
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
            field_row(ui, "시간", format!("{} {}", entry.date, entry.time), text_color, weak_text);
            ui.end_row();

            ui.label(RichText::new("레벨").color(weak_text));
            ui.label(
                RichText::new(entry.level.label())
                    .color(level_label_color(entry.level, dark_mode))
                    .monospace(),
            );
            ui.end_row();

            field_row(ui, "태그", entry.tag.clone(), text_color, weak_text);
            ui.end_row();

            field_row(ui, "PID", entry.pid.to_string(), text_color, weak_text);
            ui.end_row();

            field_row(ui, "TID", entry.tid.to_string(), text_color, weak_text);
            ui.end_row();
        });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    ui.label(RichText::new("메시지").color(weak_text));
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .max_height(180.0)
        .id_source("detail_msg_scroll")
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(&entry.message).monospace().color(text_color),
                )
                .selectable(true),
            );
        });

}

fn field_row(
    ui: &mut egui::Ui,
    label: &str,
    value: String,
    text_color: egui::Color32,
    weak_text: egui::Color32,
) {
    ui.label(RichText::new(label).color(weak_text));
    ui.label(RichText::new(value).color(text_color));
}
