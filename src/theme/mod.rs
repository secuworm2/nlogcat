pub mod colors;
pub mod font_scanner;
pub mod fonts;

use egui::{Color32, Stroke};

/// Design.md §7.1 기준 다크 테마 Visuals를 반환한다.
#[must_use]
pub fn dark_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.window_fill = Color32::from_rgb(26, 26, 26);
    v.panel_fill = Color32::from_rgb(36, 36, 36);
    v.faint_bg_color = Color32::from_rgb(46, 46, 46);
    v.override_text_color = Some(Color32::from_rgb(232, 232, 232));
    v.widgets.noninteractive.bg_fill = Color32::from_rgb(36, 36, 36);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(58, 58, 58));
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(58, 58, 58));
    v.widgets.inactive.bg_fill = Color32::from_rgb(46, 46, 46);
    v.widgets.hovered.bg_fill = Color32::from_rgb(56, 56, 56);
    v.widgets.active.bg_fill = Color32::from_rgb(58, 138, 224);
    v.selection.bg_fill = Color32::from_rgb(42, 58, 82);
    v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(77, 158, 248));
    v
}

/// Design.md §1.2 기준 라이트 테마 Visuals를 반환한다.
#[must_use]
pub fn light_visuals() -> egui::Visuals {
    let mut v = egui::Visuals::light();
    v.window_fill = Color32::from_rgb(240, 240, 240);
    v.panel_fill = Color32::WHITE;
    v.faint_bg_color = Color32::from_rgb(250, 250, 250);
    v.override_text_color = Some(Color32::from_rgb(26, 26, 26));
    v.widgets.noninteractive.bg_fill = Color32::WHITE;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(58, 58, 58));
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(58, 58, 58));
    v.widgets.inactive.bg_fill = Color32::from_rgb(250, 250, 250);
    v.widgets.hovered.bg_fill = Color32::from_rgb(228, 228, 228);
    v.selection.bg_fill = Color32::from_rgb(219, 190, 254);
    v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(77, 158, 248));
    v
}
