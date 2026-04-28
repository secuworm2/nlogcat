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

/// `NotoSansKR` 없을 때 Windows 시스템 한글 폰트 경로 후보
fn korean_font_fallback_paths() -> &'static [&'static str] {
    &[
        "assets/fonts/NotoSansKR-Regular.ttf",
        r"C:\Windows\Fonts\malgun.ttf",
        r"C:\Windows\Fonts\gulim.ttc",
    ]
}

/// 폰트 파일을 런타임에 로드해 `FontDefinitions`를 구성한다.
/// 파일이 없거나 유효하지 않으면 시스템 한글 폰트 → egui 기본 폰트 순으로 fallback한다.
#[must_use]
pub fn build_font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    let jetbrains = std::fs::read("assets/fonts/JetBrainsMono-Regular.ttf")
        .ok()
        .filter(|b| is_valid_font(b));

    // NotoSansKR 우선, 없으면 시스템 한글 폰트로 fallback
    let korean = korean_font_fallback_paths()
        .iter()
        .find_map(|path| std::fs::read(path).ok().filter(|b| is_valid_font(b)));

    if let Some(data) = jetbrains {
        fonts
            .font_data
            .insert("JetBrainsMono".to_owned(), egui::FontData::from_owned(data));
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.insert(0, "JetBrainsMono".to_owned());
        }
    }

    if let Some(data) = korean {
        fonts
            .font_data
            .insert("KoreanFont".to_owned(), egui::FontData::from_owned(data));
        // 모노스페이스/프로포셔널 패밀리에 한글 폰트를 fallback으로 추가
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.push("KoreanFont".to_owned());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.push("KoreanFont".to_owned());
        }
    }

    fonts
}
