use std::collections::HashMap;

use egui::{Color32, FontId, Frame, Margin, Rounding, Sense, Stroke, TextureHandle};

use crate::app::AppState;
use crate::theme::colors::PRIMARY;

const ICON_SIZE: f32 = 22.0;
const ROW_HEIGHT: f32 = 30.0;

pub fn render(
    ctx: &egui::Context,
    state: &mut AppState,
    icon_textures: &HashMap<String, TextureHandle>,
) {
    if !state.show_package_filter {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.show_package_filter = false;
        return;
    }

    let mut packages: Vec<String> = state
        .pid_map
        .values()
        .filter(|p| !p.is_empty())
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    packages.sort();

    let screen_rect = ctx.screen_rect();
    let default_pos = egui::pos2(8.0, 88.0);
    let _ = screen_rect;

    let window_fill = ctx.style().visuals.window_fill;
    let border_color = ctx.style().visuals.window_stroke.color;
    let mut close = false;

    egui::Window::new("__package_filter")
        .title_bar(false)
        .resizable(false)
        .default_pos(default_pos)
        .min_width(300.0)
        .frame(
            Frame::none()
                .fill(window_fill)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(8.0))
                .stroke(Stroke::new(1.0, border_color)),
        )
        .show(ctx, |ui| {
            let text_color = ui.visuals().text_color();

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("앱 필터").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        close = true;
                    }
                    if state.filter.selected_package.is_some() {
                        if ui.small_button("초기화").clicked() {
                            state.filter.selected_package = None;
                            state.filter_dirty = true;
                            close = true;
                        }
                    }
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
                    if package_row(ui, "전체 앱", None, all_sel, None, text_color).clicked() {
                        state.filter.selected_package = None;
                        state.filter_dirty = true;
                        close = true;
                    }

                    for pkg in &packages {
                        let is_sel =
                            state.filter.selected_package.as_deref() == Some(pkg.as_str());
                        let tex = icon_textures.get(pkg.as_str());
                        if package_row(ui, pkg, Some(pkg.as_str()), is_sel, tex, text_color)
                            .clicked()
                        {
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

    if close {
        state.show_package_filter = false;
    }
}

fn package_row(
    ui: &mut egui::Ui,
    label: &str,
    pkg: Option<&str>,
    is_selected: bool,
    texture: Option<&TextureHandle>,
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

    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.min.x + 4.0 + ICON_SIZE / 2.0, rect.center().y),
        egui::vec2(ICON_SIZE, ICON_SIZE),
    );

    if let Some(tex) = texture {
        ui.painter().image(
            tex.id(),
            icon_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else if let Some(pkg_name) = pkg {
        paint_avatar(ui.painter(), icon_rect, pkg_name);
    } else {
        paint_all_dots(ui.painter(), icon_rect);
    }

    let label_x = icon_rect.max.x + 8.0;
    let label_color = if is_selected { PRIMARY } else { text_color };
    ui.painter()
        .with_clip_rect(egui::Rect::from_min_max(
            egui::pos2(label_x, rect.min.y),
            egui::pos2(rect.max.x - 4.0, rect.max.y),
        ))
        .text(
            egui::pos2(label_x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.0),
            label_color,
        );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn paint_avatar(painter: &egui::Painter, rect: egui::Rect, pkg: &str) {
    let letter = pkg
        .split('.')
        .last()
        .and_then(|s| s.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?');

    let hash: usize = pkg.bytes().map(|b| b as usize).sum();
    const PALETTE: &[Color32] = &[
        Color32::from_rgb(77, 158, 248),
        Color32::from_rgb(74, 222, 128),
        Color32::from_rgb(248, 113, 113),
        Color32::from_rgb(167, 139, 250),
        Color32::from_rgb(251, 146, 60),
        Color32::from_rgb(45, 212, 191),
        Color32::from_rgb(236, 72, 153),
        Color32::from_rgb(134, 199, 86),
    ];
    let bg = PALETTE[hash % PALETTE.len()];

    painter.circle_filled(rect.center(), rect.width() / 2.0, bg);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        letter.to_string(),
        FontId::proportional(11.0),
        Color32::WHITE,
    );
}

fn paint_all_dots(painter: &egui::Painter, rect: egui::Rect) {
    let gray = Color32::from_rgb(150, 150, 150);
    let c = rect.center();
    let r = rect.width() * 0.18;
    let gap = rect.width() * 0.28;
    for dx in [-1.0_f32, 1.0] {
        for dy in [-1.0_f32, 1.0] {
            painter.circle_filled(egui::pos2(c.x + dx * gap, c.y + dy * gap), r, gray);
        }
    }
}
