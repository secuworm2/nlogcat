use crate::app::AppState;

pub fn render(ctx: &egui::Context, _state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |_ui| {});
}
