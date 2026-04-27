/// 폰트 파일을 런타임에 로드해 `FontDefinitions`를 구성한다.
/// 파일이 없거나 비어있으면 egui 기본 폰트로 graceful fallback한다.
#[must_use]
pub fn build_font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    let jetbrains = std::fs::read("assets/fonts/JetBrainsMono-Regular.ttf")
        .ok()
        .filter(|b| !b.is_empty());

    let noto = std::fs::read("assets/fonts/NotoSansKR-Regular.ttf")
        .ok()
        .filter(|b| !b.is_empty());

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
