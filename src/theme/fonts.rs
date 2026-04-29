fn is_valid_font(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    matches!(
        data[..4],
        [0x00, 0x01, 0x00, 0x00]
        | [0x4F, 0x54, 0x54, 0x4F]
        | [0x74, 0x74, 0x63, 0x66]
    )
}

fn load_path(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok().filter(|b| is_valid_font(b))
}

fn load_korean() -> Option<Vec<u8>> {
    [
        "assets/fonts/NotoSansKR-Regular.ttf",
        r"C:\Windows\Fonts\malgun.ttf",
        r"C:\Windows\Fonts\gulim.ttc",
    ]
    .iter()
    .find_map(|p| load_path(p))
}

fn load_mono(family: &str) -> Option<Vec<u8>> {
    match family {
        "JetBrainsMono" => load_path("assets/fonts/JetBrainsMono-Regular.ttf"),
        "Consolas" => load_path(r"C:\Windows\Fonts\consola.ttf"),
        "CascadiaCode" => load_path(r"C:\Windows\Fonts\CascadiaCode.ttf")
            .or_else(|| load_path(r"C:\Windows\Fonts\CascadiaMono.ttf")),
        _ => None,
    }
}

/// Returns the font families selectable in settings (key, display label).
/// All options are always offered; missing font files fall back silently.
pub const FONT_OPTIONS: &[(&str, &str)] = &[
    ("JetBrainsMono", "JetBrains Mono"),
    ("Consolas", "Consolas"),
    ("CascadiaCode", "Cascadia Code"),
    ("Default", "기본"),
];

#[must_use]
pub fn build_font_definitions_with_family(family: &str) -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(data) = load_mono(family) {
        fonts
            .font_data
            .insert(family.to_owned(), egui::FontData::from_owned(data));
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            f.insert(0, family.to_owned());
        }
    }

    if let Some(data) = load_korean() {
        fonts
            .font_data
            .insert("KoreanFont".to_owned(), egui::FontData::from_owned(data));
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            f.push("KoreanFont".to_owned());
        }
        if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            f.push("KoreanFont".to_owned());
        }
    }

    fonts
}

#[must_use]
pub fn build_font_definitions() -> egui::FontDefinitions {
    build_font_definitions_with_family("JetBrainsMono")
}
