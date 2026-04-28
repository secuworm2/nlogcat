use egui::{Color32, FontId, Margin, Rounding, Stroke};

use crate::app::AppState;
use crate::theme::colors::{BG_ELEVATED, BORDER_DEFAULT};

// Design.md §4.8 경고 배너 색상
const WARNING_BG: Color32 = Color32::from_rgb(45, 26, 10);
const WARNING_ACCENT: Color32 = Color32::from_rgb(245, 158, 11);
const WARNING_TEXT: Color32 = Color32::from_rgb(252, 211, 77);

pub fn render(ctx: &egui::Context, state: &mut AppState) {
    // ADB 미설치 경고 배너 (조건부 표시)
    if state.adb_error.is_some() {
        egui::TopBottomPanel::bottom("adb_warning_banner")
            .show_separator_line(false)
            .show(ctx, |ui| {
                if let Some(ref err) = state.adb_error {
                    render_warning_banner(ui, err);
                }
            });
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_size();
        let card_approx_height = 210.0;
        let v_pad = ((available.y - card_approx_height) / 2.0).max(8.0);

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(v_pad);

            egui::Frame::none()
                .fill(BG_ELEVATED)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(24.0))
                .stroke(Stroke::new(1.0, BORDER_DEFAULT))
                .show(ui, |ui| {
                    ui.set_width(300.0);
                    render_card(ui, state);
                });
        });
    });
}

fn render_card(ui: &mut egui::Ui, state: &mut AppState) {
    // 디바이스 정보를 미리 수집 (borrow 충돌 방지)
    let devices_info: Vec<(String, String)> = state
        .devices
        .iter()
        .map(|d| {
            let label = d
                .model
                .as_deref()
                .map(|m| format!("{} ({})", m, d.serial))
                .unwrap_or_else(|| d.serial.clone());
            (d.serial.clone(), label)
        })
        .collect();

    let has_devices = !devices_info.is_empty();

    let selected_label = state
        .selected_device
        .as_deref()
        .and_then(|serial| devices_info.iter().find(|(s, _)| s == serial))
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| {
            if has_devices {
                "디바이스 선택...".to_string()
            } else {
                "디바이스 없음".to_string()
            }
        });

    ui.vertical(|ui| {
        ui.heading("디바이스를 연결하세요");
        ui.add_space(8.0);
        ui.label("USB 디버깅 활성화 후\nAndroid 기기를 연결해 주세요.");
        ui.add_space(16.0);

        // 디바이스 드롭다운 — 목록 없으면 비활성
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
                    state.selected_device = new_selection;
                });
        });

        ui.add_space(8.0);

        // 새로고침 버튼 — DevicePoller에 즉시 재조회 요청
        if ui.button("새로고침").clicked() {
            let _ = state.device_poll_tx.try_send(());
        }
    });
}

fn render_warning_banner(ui: &mut egui::Ui, message: &str) {
    let available_width = ui.available_width();
    let text_height = ui.text_style_height(&egui::TextStyle::Body);
    let total_height = text_height + 20.0;

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(available_width, total_height),
        egui::Sense::hover(),
    );

    let painter = ui.painter();

    // 배경: #2D1A0A
    painter.rect_filled(rect, 0.0, WARNING_BG);

    // 좌측 강조선: #F59E0B 3px
    let accent_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.min.x + 3.0, rect.max.y),
    );
    painter.rect_filled(accent_rect, 0.0, WARNING_ACCENT);

    // 텍스트: #FCD34D
    painter.text(
        egui::pos2(rect.min.x + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("⚠  {message}"),
        FontId::default(),
        WARNING_TEXT,
    );
}
