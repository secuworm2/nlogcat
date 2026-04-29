use egui::{Color32, FontId, Frame, Margin};

use crate::app::AppState;
use crate::theme::colors::BG_SURFACE;

const ERROR_BG: Color32 = Color32::from_rgb(45, 26, 10);
const ERROR_ACCENT: Color32 = Color32::from_rgb(245, 158, 11);
const ERROR_TEXT: Color32 = Color32::from_rgb(252, 211, 77);

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("toolbar")
        .exact_height(40.0)
        .frame(Frame::none().fill(BG_SURFACE))
        .show(ctx, |ui| {
            crate::ui::toolbar::render(ui, state);
        });

    if state.last_error.is_some() {
        egui::TopBottomPanel::top("error_banner")
            .exact_height(28.0)
            .frame(Frame::none().fill(ERROR_BG).inner_margin(Margin::ZERO))
            .show_separator_line(false)
            .show(ctx, |ui| {
                if let Some(ref msg) = state.last_error {
                    render_error_banner(ui, msg);
                }
            });
    }

    egui::TopBottomPanel::top("filter_bar")
        .exact_height(40.0)
        .frame(Frame::none().fill(BG_SURFACE))
        .show(ctx, |ui| {
            crate::ui::filter_bar::render(ui, state);
        });

    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(24.0)
        .frame(Frame::none().fill(BG_SURFACE))
        .show(ctx, |ui| {
            crate::ui::status_bar::render(ui, state);
        });

    crate::ui::detail_panel::render(ctx, state);

    egui::CentralPanel::default().show(ctx, |ui| {
        crate::ui::log_table::render(ui, state);
    });

    // Modal overlays — rendered after panels so they appear on top
    crate::ui::detail_modal::render(ctx, state);
    crate::ui::settings_panel::render(ctx, state);
}

fn render_error_banner(ui: &mut egui::Ui, message: &str) {
    let available_width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(available_width, 28.0),
        egui::Sense::hover(),
    );

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, ERROR_BG);

    let accent_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.min.x + 3.0, rect.max.y),
    );
    painter.rect_filled(accent_rect, 0.0, ERROR_ACCENT);

    painter.text(
        egui::pos2(rect.min.x + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("⚠  {message}"),
        FontId::default(),
        ERROR_TEXT,
    );
}
