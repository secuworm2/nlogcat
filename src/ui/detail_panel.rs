use egui::{Frame, Margin, RichText, Stroke};

use crate::app::AppState;
use crate::theme::colors::{
    level_label_color, BG_SURFACE, BORDER_DEFAULT, TEXT_PRIMARY, TEXT_SECONDARY,
};

pub fn render(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("detail_panel")
        .resizable(true)
        .min_height(60.0)
        .default_height(110.0)
        .frame(
            Frame::none()
                .fill(BG_SURFACE)
                .inner_margin(Margin::symmetric(8.0, 6.0))
                .stroke(Stroke::new(1.0, BORDER_DEFAULT)),
        )
        .show_separator_line(false)
        .show(ctx, |ui| {
            let entry = state.focused_log_id.and_then(|id| {
                let Ok(buf) = state.log_buffer.lock() else { return None; };
                buf.find_by_id(id).cloned()
            });

            match entry {
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("로그를 선택하면 메시지가 표시됩니다")
                                .color(TEXT_SECONDARY),
                        );
                    });
                }
                Some(entry) => {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{} {}", entry.date, entry.time))
                                .color(TEXT_SECONDARY)
                                .monospace(),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(entry.level.label())
                                .color(level_label_color(entry.level))
                                .monospace(),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(&entry.tag).color(TEXT_PRIMARY).monospace(),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("PID:{}", entry.pid))
                                .color(TEXT_SECONDARY)
                                .monospace(),
                        );
                    });

                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .id_source("detail_panel_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&entry.message).monospace().color(TEXT_PRIMARY),
                                )
                                .selectable(true),
                            );
                        });
                }
            }
        });
}
