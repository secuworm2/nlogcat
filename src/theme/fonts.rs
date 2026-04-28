fn is_valid_font(data: &[u8]) -> bool {
    // TTF/OTF 매직 바이트 확인으로 유효하지 않은 placeholder 파일을 필터링한다
    if data.len() < 4 {
        return false;
    }
    matches!(
        data[..4],
        [0x00, 0x01, 0x00, 0x00]  // TrueType
        | [0x4F, 0x54, 0x54, 0x4F]  // OpenType CFF ("OTTO")
        | [0x74, 0x74, 0x63, 0x66]  // TrueType Collection ("ttcf")
    )
}

/// 폰트 파일을 런타임에 로드해 `FontDefinitions`를 구성한다.
/// 파일이 없거나 유효하지 않으면 egui 기본 폰트로 graceful fallback한다.
#[must_use]
pub fn build_font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    let jetbrains = std::fs::read("assets/fonts/JetBrainsMono-Regular.ttf")
        .ok()
        .filter(|b| is_valid_font(b));

    let noto = std::fs::read("assets/fonts/NotoSansKR-Regular.ttf")
        .ok()
        .filter(|b| is_valid_font(b));

    if let Some(data) = jetbrains {
        fonts
            .font_data
            .insert("JetBrainsMono".to_owned(), egui::FontData::from_owned(data));
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.insert(0, "JetBrainsMono".to_owned());
        }
    }

    if let Some(data) = noto {
        fonts
            .font_data
            .insert("NotoSansKR".to_owned(), egui::FontData::from_owned(data));
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.push("NotoSansKR".to_owned());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.push("NotoSansKR".to_owned());
        }
    }

    fonts
}
