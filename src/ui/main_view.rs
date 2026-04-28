use egui::Frame;

use crate::app::AppState;
use crate::theme::colors::BG_SURFACE;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("toolbar")
        .exact_height(40.0)
        .frame(Frame::none().fill(BG_SURFACE))
        .show(ctx, |ui| {
            crate::ui::toolbar::render(ui, state);
        });

    egui::TopBottomPanel::top("filter_bar")
        .exact_height(56.0)
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

    egui::CentralPanel::default().show(ctx, |ui| {
        crate::ui::log_table::render(ui, state);
    });
}
