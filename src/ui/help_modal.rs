use egui::{Frame, Margin, RichText, Rounding, Stroke};

use crate::app::AppState;

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_help {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.show_help = false;
        return;
    }

    let window_fill = ctx.style().visuals.window_fill;
    let border_color = ctx.style().visuals.window_stroke.color;
    let mut close = false;

    let use_center = state.help_just_opened;
    state.help_just_opened = false;

    let screen_rect = ctx.screen_rect();
    let default_pos = screen_rect.center() - egui::vec2(186.0, 190.0);

    let mut window = egui::Window::new("__help_modal")
        .title_bar(false)
        .resizable(false)
        .default_pos(default_pos)
        .min_width(340.0)
        .frame(
            Frame::none()
                .fill(window_fill)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(16.0))
                .stroke(Stroke::new(1.0, border_color)),
        );

    if use_center {
        window = window.current_pos(default_pos);
    }

    window.show(ctx, |ui| {
        render_content(ui, &mut close);
    });

    if close {
        state.show_help = false;
    }
}

fn render_content(ui: &mut egui::Ui, close: &mut bool) {
    let text_color = ui.visuals().text_color();
    let weak_text = ui.visuals().weak_text_color();

    ui.horizontal(|ui| {
        ui.label(RichText::new("도움말").strong().color(text_color));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("X").clicked() {
                *close = true;
            }
        });
    });

    ui.separator();
    ui.add_space(6.0);

    ui.label(RichText::new("단축키").strong().size(11.0).color(weak_text));
    ui.add_space(4.0);

    let shortcuts: &[(&str, &str)] = &[
        ("↑ / ↓",        "이전/다음 로그 선택"),
        ("더블클릭",       "로그 상세 보기"),
        ("Ctrl + A",      "전체 선택"),
        ("Delete",        "선택된 로그 삭제"),
        ("Ctrl + F",      "검색어 입력 필드 포커스"),
        ("Ctrl + 클릭",   "다중 선택"),
        ("Shift + 클릭",  "범위 선택"),
        ("Ctrl + 스크롤", "폰트 크기 조절"),
        ("Esc",           "팝업 닫기"),
    ];

    egui::Grid::new("help_shortcuts")
        .num_columns(2)
        .spacing([24.0, 5.0])
        .show(ui, |ui| {
            for &(key, desc) in shortcuts {
                ui.label(RichText::new(key).monospace().color(text_color));
                ui.label(RichText::new(desc).color(weak_text));
                ui.end_row();
            }
        });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);

    ui.label(
        RichText::new("Copyright © secuworm. Released under the MIT License.")
            .color(weak_text)
            .size(11.0),
    );

    ui.add_space(8.0);
}
