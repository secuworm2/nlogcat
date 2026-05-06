use egui::{Color32, FontId, Frame, Margin, Order, Rounding, Sense, Stroke};

use crate::app::AppState;
use crate::theme::colors::PRIMARY;

const ROW_HEIGHT: f32 = 28.0;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_package_filter {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.show_package_filter = false;
        return;
    }

    let packages: Vec<String> = state
        .active_packages
        .iter()
        .filter(|p| {
            if state.filter.show_system_apps || state.user_packages.is_empty() {
                true
            } else {
                state.user_packages.contains(p.as_str())
            }
        })
        .cloned()
        .collect();

    let window_fill = ctx.style().visuals.window_fill;
    let border_color = ctx.style().visuals.window_stroke.color;
    let mut close = false;

    let anchor = state.package_filter_anchor;

    egui::Area::new("__package_filter".into())
        .fixed_pos(anchor)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::none()
                .fill(window_fill)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(8.0))
                .stroke(Stroke::new(1.0, border_color))
                .show(ui, |ui| {
                    ui.set_min_width(320.0);
                    let text_color = ui.visuals().text_color();

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("앱 목록").strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("X").clicked() {
                                close = true;
                            }
                            if state.filter.selected_package.is_some() {
                                if ui.small_button("초기화").clicked() {
                                    state.filter.selected_package = None;
                                    state.filter_dirty = true;
                                    close = true;
                                }
                            }
                            ui.checkbox(&mut state.filter.show_system_apps, "시스템 앱 표시");
                        });
                    });

                    ui.add_space(4.0);
                    ui.painter().hline(
                        ui.max_rect().x_range(),
                        ui.cursor().min.y,
                        Stroke::new(1.0, border_color),
                    );
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .max_height(320.0)
                        .id_source("pkg_filter_scroll")
                        .show(ui, |ui| {
                            let all_sel = state.filter.selected_package.is_none();
                            if package_row(ui, "전체 앱", None, all_sel, text_color).clicked() {
                                state.filter.selected_package = None;
                                state.filter_dirty = true;
                                close = true;
                            }

                            for pkg in &packages {
                                let is_sel = state.filter.selected_package.as_deref()
                                    == Some(pkg.as_str());
                                let label = state.app_labels.get(pkg.as_str()).map(String::as_str);
                                if package_row(ui, pkg, label, is_sel, text_color).clicked() {
                                    state.filter.selected_package = Some(pkg.clone());
                                    state.filter_dirty = true;
                                    close = true;
                                }
                            }

                            if packages.is_empty() {
                                ui.add_space(8.0);
                                ui.centered_and_justified(|ui| {
                                    ui.weak("실행 중인 앱 없음");
                                });
                                ui.add_space(8.0);
                            }
                        });
                });
        });

    if close {
        state.show_package_filter = false;
    }
}

fn package_row(
    ui: &mut egui::Ui,
    pkg_id: &str,
    app_label: Option<&str>,
    is_selected: bool,
    text_color: Color32,
) -> egui::Response {
    let avail_w = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(avail_w, ROW_HEIGHT), Sense::click());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let bg = if is_selected {
        ui.visuals().selection.bg_fill
    } else if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, 3.0, bg);
    }

    let label_color = if is_selected { PRIMARY } else { text_color };
    let display_text = if let Some(label) = app_label {
        format!("{label} ({pkg_id})")
    } else {
        pkg_id.to_owned()
    };

    ui.painter()
        .with_clip_rect(egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 6.0, rect.min.y),
            egui::pos2(rect.max.x - 6.0, rect.max.y),
        ))
        .text(
            egui::pos2(rect.min.x + 6.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            display_text,
            FontId::proportional(12.0),
            label_color,
        );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
