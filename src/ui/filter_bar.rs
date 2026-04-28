use egui::{Color32, RichText, Stroke, TextEdit};

use crate::app::AppState;
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

        let search_resp = ui.add(
            TextEdit::singleline(&mut state.filter.search_query)
                .hint_text("검색어 입력")
                .desired_width(160.0),
        );
        if search_resp.changed() {
            // Debounce: set filter_dirty after 150ms to avoid recompute on every keystroke
            state.search_debounce_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(150));
        }

        ui.add_space(4.0);

        if case_button(ui, state.filter.case_sensitive).clicked() {
            state.filter.case_sensitive = !state.filter.case_sensitive;
            state.filter_dirty = true;
        }

        ui.add_space(8.0);

        // Tag input: raw string stored in egui temp memory, parsed to Vec<String> on change
        let tag_id = egui::Id::new("filter_bar_tag");
        let mut tag_str: String = ui
            .data_mut(|d| d.get_temp::<String>(tag_id))
            .unwrap_or_else(|| state.filter.tag_includes.join(", "));
        let tag_resp = ui.add(
            TextEdit::singleline(&mut tag_str)
                .hint_text("태그 (콤마 구분)")
                .desired_width(140.0),
        );
        if tag_resp.changed() {
            state.filter.tag_includes = tag_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            state.filter_dirty = true;
        }
        ui.data_mut(|d| d.insert_temp(tag_id, tag_str));

        ui.add_space(8.0);

        // PID input: raw string stored in egui temp memory, digits only, parsed to Option<u32>
        let pid_id = egui::Id::new("filter_bar_pid");
        let mut pid_str: String = ui
            .data_mut(|d| d.get_temp::<String>(pid_id))
            .unwrap_or_else(|| {
                state
                    .filter
                    .pid_filter
                    .map_or(String::new(), |p| p.to_string())
            });
        let pid_resp = ui.add(
            TextEdit::singleline(&mut pid_str)
                .hint_text("PID")
                .desired_width(70.0),
        );
        if pid_resp.changed() {
            pid_str.retain(|c| c.is_ascii_digit());
            state.filter.pid_filter = if pid_str.is_empty() {
                None
            } else {
                pid_str.parse::<u32>().ok()
            };
            state.filter_dirty = true;
        }
        ui.data_mut(|d| d.insert_temp(pid_id, pid_str));
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
            let bg = Color32::from_rgba_unmultiplied(r, g, b, 51); // 20% opacity
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
            egui::Button::new(RichText::new("Aa").color(if active {
                PRIMARY
            } else {
                TEXT_SECONDARY
            }))
            .fill(Color32::TRANSPARENT),
        )
    })
    .inner
}
