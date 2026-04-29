use egui::{Color32, ComboBox, RichText, Stroke, TextEdit};

use crate::app::AppState;
use crate::model::filter_state::SearchField;
use crate::model::LogLevel;
use crate::theme::colors::{
    level_label_color, BG_HOVER, BORDER_DEFAULT, PRIMARY, TEXT_PRIMARY, TEXT_SECONDARY,
};

const LEVELS: &[(LogLevel, &str)] = &[
    (LogLevel::Verbose, "V"),
    (LogLevel::Debug, "D"),
    (LogLevel::Info, "I"),
    (LogLevel::Warn, "W"),
    (LogLevel::Error, "E"),
    (LogLevel::Fatal, "F"),
];

const SEARCH_FIELDS: &[SearchField] = &[
    SearchField::All,
    SearchField::Tag,
    SearchField::Pid,
    SearchField::Package,
    SearchField::Message,
];

pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal_centered(|ui| {
        ui.add_space(8.0);

        for &(level, label) in LEVELS {
            let active = state.filter.levels.contains(&level);
            if level_toggle(ui, label, level, active).clicked() {
                if active {
                    state.filter.levels.remove(&level);
                } else {
                    state.filter.levels.insert(level);
                }
                state.filter_dirty = true;
            }
            ui.add_space(2.0);
        }

        ui.add_space(8.0);

        ComboBox::from_id_source("search_field_combo")
            .selected_text(state.filter.search_field.label())
            .width(68.0)
            .show_ui(ui, |ui| {
                for field in SEARCH_FIELDS {
                    let selected = state.filter.search_field == *field;
                    if ui.selectable_label(selected, field.label()).clicked() && !selected {
                        state.filter.search_field = field.clone();
                        if !state.filter.search_query.is_empty() {
                            state.filter_dirty = true;
                        }
                    }
                }
            });

        ui.add_space(4.0);

        let search_resp = ui.add(
            TextEdit::singleline(&mut state.filter.search_query)
                .hint_text("검색어 입력")
                .desired_width(220.0),
        );
        if search_resp.changed() {
            state.search_debounce_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(150));
        }

        ui.add_space(4.0);

        if case_button(ui, state.filter.case_sensitive).clicked() {
            state.filter.case_sensitive = !state.filter.case_sensitive;
            state.filter_dirty = true;
        }
    });
}

fn level_toggle(
    ui: &mut egui::Ui,
    label: &str,
    level: LogLevel,
    active: bool,
) -> egui::Response {
    let color = level_label_color(level);
    ui.scope(|ui| {
        let w = &mut ui.style_mut().visuals.widgets;
        if active {
            let [r, g, b, _] = color.to_array();
            let bg = Color32::from_rgba_unmultiplied(r, g, b, 51);
            w.inactive.weak_bg_fill = bg;
            w.inactive.bg_fill = bg;
            w.inactive.fg_stroke = Stroke::new(1.0, color);
            w.inactive.bg_stroke = Stroke::new(1.0, color);
            w.hovered.weak_bg_fill = bg;
            w.hovered.bg_fill = bg;
            w.hovered.fg_stroke = Stroke::new(1.0, color);
            w.hovered.bg_stroke = Stroke::new(1.0, color);
            w.active.weak_bg_fill = bg;
            w.active.bg_fill = bg;
            w.active.fg_stroke = Stroke::new(1.0, color);
            w.active.bg_stroke = Stroke::new(1.0, color);
        } else {
            w.inactive.weak_bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_fill = Color32::TRANSPARENT;
            w.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
            w.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
            w.hovered.weak_bg_fill = BG_HOVER;
            w.hovered.bg_fill = BG_HOVER;
            w.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
            w.hovered.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
            w.active.weak_bg_fill = BG_HOVER;
            w.active.bg_fill = BG_HOVER;
            w.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
            w.active.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        }
        ui.button(label)
    })
    .inner
}

fn case_button(ui: &mut egui::Ui, active: bool) -> egui::Response {
    ui.scope(|ui| {
        let w = &mut ui.style_mut().visuals.widgets;
        if active {
            w.inactive.weak_bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_fill = Color32::TRANSPARENT;
            w.inactive.fg_stroke = Stroke::new(1.0, PRIMARY);
            w.inactive.bg_stroke = Stroke::new(1.0, PRIMARY);
            w.hovered.weak_bg_fill = BG_HOVER;
            w.hovered.bg_fill = BG_HOVER;
            w.hovered.fg_stroke = Stroke::new(1.0, PRIMARY);
            w.hovered.bg_stroke = Stroke::new(1.0, PRIMARY);
            w.active.weak_bg_fill = BG_HOVER;
            w.active.bg_fill = BG_HOVER;
            w.active.fg_stroke = Stroke::new(1.0, PRIMARY);
            w.active.bg_stroke = Stroke::new(1.0, PRIMARY);
        } else {
            w.inactive.weak_bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_fill = Color32::TRANSPARENT;
            w.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
            w.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
            w.hovered.weak_bg_fill = BG_HOVER;
            w.hovered.bg_fill = BG_HOVER;
            w.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
            w.hovered.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
            w.active.weak_bg_fill = BG_HOVER;
            w.active.bg_fill = BG_HOVER;
            w.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
            w.active.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        }
        ui.add(
            egui::Button::new(
                RichText::new("Aa").color(if active { PRIMARY } else { TEXT_SECONDARY }),
            )
            .fill(Color32::TRANSPARENT),
        )
    })
    .inner
}
