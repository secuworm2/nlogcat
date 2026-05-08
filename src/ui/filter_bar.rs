use egui::{Color32, ComboBox, RichText, Stroke, TextEdit};

use crate::app::AppState;
use crate::model::filter_state::SearchField;
use crate::model::{LogLevel, Platform};
use crate::theme::colors::{level_label_color, PRIMARY};

const LEVELS: &[LogLevel] = &[
    LogLevel::Verbose,
    LogLevel::Debug,
    LogLevel::Info,
    LogLevel::Warn,
    LogLevel::Error,
    LogLevel::Fatal,
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

        let dark_mode = ui.visuals().dark_mode;
        let platform = state.current_platform();
        for &level in LEVELS {
            let label = match platform {
                Platform::Android => level.label(),
                Platform::Ios => level.ios_label(),
            };
            let active = state.filter.levels.contains(&level);
            if level_toggle(ui, label, level, active, dark_mode).clicked() {
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
        if state.focus_search {
            search_resp.request_focus();
            state.focus_search = false;
        }
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
    dark_mode: bool,
) -> egui::Response {
    let color = level_label_color(level, dark_mode);
    ui.scope(|ui| {
        // Extract visuals before taking mutable borrow of style
        let weak_color = ui.visuals().weak_text_color();
        let hover_bg = ui.visuals().widgets.hovered.bg_fill;
        let text_color = ui.visuals().text_color();


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
            w.inactive.fg_stroke = Stroke::new(1.0, weak_color);
            w.inactive.bg_stroke = Stroke::NONE;
            w.hovered.weak_bg_fill = hover_bg;
            w.hovered.bg_fill = hover_bg;
            w.hovered.fg_stroke = Stroke::new(1.0, text_color);
            w.hovered.bg_stroke = Stroke::NONE;
            w.active.weak_bg_fill = hover_bg;
            w.active.bg_fill = hover_bg;
            w.active.fg_stroke = Stroke::new(1.0, text_color);
            w.active.bg_stroke = Stroke::NONE;
        }
        ui.button(label)
    })
    .inner
}

fn case_button(ui: &mut egui::Ui, active: bool) -> egui::Response {
    ui.scope(|ui| {
        // Extract visuals before taking mutable borrow of style
        let weak_color = ui.visuals().weak_text_color();
        let hover_bg = ui.visuals().widgets.hovered.bg_fill;
        let text_color = ui.visuals().text_color();


        let w = &mut ui.style_mut().visuals.widgets;
        if active {
            w.inactive.weak_bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_fill = Color32::TRANSPARENT;
            w.inactive.fg_stroke = Stroke::new(1.0, PRIMARY);
            w.inactive.bg_stroke = Stroke::new(1.0, PRIMARY);
            w.hovered.weak_bg_fill = hover_bg;
            w.hovered.bg_fill = hover_bg;
            w.hovered.fg_stroke = Stroke::new(1.0, PRIMARY);
            w.hovered.bg_stroke = Stroke::new(1.0, PRIMARY);
            w.active.weak_bg_fill = hover_bg;
            w.active.bg_fill = hover_bg;
            w.active.fg_stroke = Stroke::new(1.0, PRIMARY);
            w.active.bg_stroke = Stroke::new(1.0, PRIMARY);
        } else {
            w.inactive.weak_bg_fill = Color32::TRANSPARENT;
            w.inactive.bg_fill = Color32::TRANSPARENT;
            w.inactive.fg_stroke = Stroke::new(1.0, weak_color);
            w.inactive.bg_stroke = Stroke::NONE;
            w.hovered.weak_bg_fill = hover_bg;
            w.hovered.bg_fill = hover_bg;
            w.hovered.fg_stroke = Stroke::new(1.0, text_color);
            w.hovered.bg_stroke = Stroke::NONE;
            w.active.weak_bg_fill = hover_bg;
            w.active.bg_fill = hover_bg;
            w.active.fg_stroke = Stroke::new(1.0, text_color);
            w.active.bg_stroke = Stroke::NONE;
        }
        ui.add(
            egui::Button::new(
                RichText::new("Aa").color(if active { PRIMARY } else { weak_color }),
            )
            .fill(Color32::TRANSPARENT),
        )
    })
    .inner
}
