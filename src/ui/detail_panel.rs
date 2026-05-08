use egui::{Frame, Margin, RichText, Stroke};

use crate::app::AppState;
use crate::model::Platform;
use crate::theme::colors::level_label_color;

pub fn render(ctx: &egui::Context, state: &AppState) {
    let panel_fill = ctx.style().visuals.panel_fill;
    let border_color = ctx.style().visuals.widgets.noninteractive.bg_stroke.color;

    egui::TopBottomPanel::bottom("detail_panel")
        .resizable(true)
        .min_height(60.0)
        .default_height(110.0)
        .frame(
            Frame::none()
                .fill(panel_fill)
                .inner_margin(Margin::symmetric(8.0, 6.0))
                .stroke(Stroke::new(1.0, border_color)),
        )
        .show_separator_line(false)
        .show(ctx, |ui| {
            let dark_mode = ui.visuals().dark_mode;
            let text_color = ui.visuals().text_color();
            let weak_text = ui.visuals().weak_text_color();

            let entry = state.focused_log_id.and_then(|id| {
                let Ok(buf) = state.log_buffer.lock() else { return None; };
                buf.find_by_id(id).cloned()
            });

            match entry {
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("로그를 선택하면 메시지가 표시됩니다")
                                .color(weak_text),
                        );
                    });
                }
                Some(entry) => {
                    let font_size = state.settings.font_size;
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{} {}", entry.date, entry.time))
                                .color(weak_text)
                                .monospace()
                                .size(font_size),
                        );
                        ui.add_space(8.0);
                        let lv_text = if state.current_platform() == Platform::Ios {
                            entry.level.ios_full_label()
                        } else {
                            entry.level.full_label()
                        };
                        ui.label(
                            RichText::new(lv_text)
                                .color(level_label_color(entry.level, dark_mode))
                                .monospace()
                                .size(font_size),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(&entry.tag).color(text_color).monospace().size(font_size),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("PID:{}", entry.pid))
                                .color(weak_text)
                                .monospace()
                                .size(font_size),
                        );
                    });

                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .id_source("detail_panel_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&entry.message)
                                        .monospace()
                                        .color(text_color)
                                        .size(font_size),
                                )
                                .selectable(true),
                            );
                        });
                }
            }
        });
}
