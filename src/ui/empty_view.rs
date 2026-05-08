use egui::{Color32, FontId, Margin, Rounding, Stroke};

use crate::app::AppState;
use crate::model::Platform;

const WARNING_BG: Color32 = Color32::from_rgb(45, 26, 10);
const WARNING_ACCENT: Color32 = Color32::from_rgb(245, 158, 11);
const WARNING_TEXT: Color32 = Color32::from_rgb(252, 211, 77);

const INFO_BG: Color32 = Color32::from_rgb(10, 30, 50);
const INFO_ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
const INFO_TEXT: Color32 = Color32::from_rgb(147, 197, 253);

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    // ADB 미설치 경고
    if state.adb_error.is_some() || (state.ios_available && !state.itunes_installed) {
        egui::TopBottomPanel::bottom("adb_warning_banner")
            .show_separator_line(false)
            .show(ctx, |ui| {
                if let Some(ref err) = state.adb_error {
                    render_banner(ui, err, WARNING_BG, WARNING_ACCENT, WARNING_TEXT);
                }
                if state.ios_available && !state.itunes_installed {
                    render_banner(
                        ui,
                        "iTunes(Apple Mobile Device Support)가 설치되어 있지 않습니다. iOS 기기 연결에 필요합니다.",
                        INFO_BG,
                        INFO_ACCENT,
                        INFO_TEXT,
                    );
                }
            });
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_size();
        let card_approx_height = 220.0;
        let v_pad = ((available.y - card_approx_height) / 2.0).max(8.0);

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(v_pad);

            let card_fill = ui.visuals().window_fill;
            let card_border = ui.visuals().window_stroke.color;

            egui::Frame::none()
                .fill(card_fill)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(24.0))
                .stroke(Stroke::new(1.0, card_border))
                .show(ui, |ui| {
                    ui.set_width(300.0);
                    render_card(ui, state);
                });
        });
    });

    crate::ui::settings_panel::render(ctx, state);
}

fn render_card(ui: &mut egui::Ui, state: &mut AppState) {
    let devices_info: Vec<(String, String)> = state
        .devices
        .iter()
        .map(|d| (d.serial.clone(), d.display_label()))
        .collect();

    let has_devices = !devices_info.is_empty();

    let selected_label = state
        .selected_device
        .as_deref()
        .and_then(|serial| devices_info.iter().find(|(s, _)| s == serial))
        .map_or_else(
            || {
                if has_devices {
                    "디바이스 선택...".to_string()
                } else {
                    "디바이스 없음".to_string()
                }
            },
            |(_, label)| label.clone(),
        );

    ui.vertical(|ui| {
        ui.heading("디바이스를 연결하세요");
        ui.add_space(8.0);

        // 선택된 디바이스 플랫폼에 따라 안내 문구 표시
        let guidance = match state.selected_device.as_deref().and_then(|s| {
            state.devices.iter().find(|d| d.serial == s).map(|d| &d.platform)
        }) {
            Some(Platform::Ios) => "[iOS] iTunes 설치 후 기기에서 신뢰를 허용해 주세요.",
            _ => "[Android] USB 디버깅 활성화 후 연결해 주세요.\n[iOS] iTunes 설치 후 연결해 주세요.",
        };
        ui.label(guidance);
        ui.add_space(16.0);

        ui.add_enabled_ui(has_devices, |ui| {
            egui::ComboBox::from_id_source("device_select_scr01")
                .width(252.0)
                .selected_text(&selected_label)
                .show_ui(ui, |ui| {
                    let mut new_selection = state.selected_device.clone();
                    for (serial, label) in &devices_info {
                        let is_selected =
                            state.selected_device.as_deref() == Some(serial.as_str());
                        if ui.selectable_label(is_selected, label).clicked() {
                            new_selection = Some(serial.clone());
                        }
                    }
                    if new_selection != state.selected_device && new_selection.is_some() {
                        state.is_streaming = true;
                    }
                    state.selected_device = new_selection;
                });
        });

        ui.add_space(8.0);

        if ui.button("새로고침").clicked() {
            let _ = state.device_poll_tx.try_send(());
        }
    });
}

fn render_banner(ui: &mut egui::Ui, message: &str, bg: Color32, accent: Color32, text_color: Color32) {
    let available_width = ui.available_width();
    let text_height = ui.text_style_height(&egui::TextStyle::Body);
    let total_height = text_height + 20.0;

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(available_width, total_height),
        egui::Sense::hover(),
    );

    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, bg);

    let accent_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.min.x + 3.0, rect.max.y),
    );
    painter.rect_filled(accent_rect, 0.0, accent);

    painter.text(
        egui::pos2(rect.min.x + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("⚠  {message}"),
        FontId::default(),
        text_color,
    );
}
