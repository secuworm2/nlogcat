use egui::Color32;

use crate::model::LogLevel;

// ── Background ────────────────────────────────────────────────────────────────
pub const BG_BASE: Color32 = Color32::from_rgb(26, 26, 26);
pub const BG_SURFACE: Color32 = Color32::from_rgb(36, 36, 36);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(46, 46, 46);
pub const BG_HOVER: Color32 = Color32::from_rgb(56, 56, 56);
pub const BG_SELECTED: Color32 = Color32::from_rgb(42, 58, 82);

// ── Border ────────────────────────────────────────────────────────────────────
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(58, 58, 58);
pub const BORDER_FOCUS: Color32 = Color32::from_rgb(77, 158, 248);

// ── Primary ───────────────────────────────────────────────────────────────────
pub const PRIMARY: Color32 = Color32::from_rgb(77, 158, 248);
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(109, 179, 255);
pub const PRIMARY_PRESSED: Color32 = Color32::from_rgb(58, 138, 224);
pub const PRIMARY_MUTED: Color32 = Color32::from_rgb(30, 58, 95);

// ── Secondary ─────────────────────────────────────────────────────────────────
pub const SECONDARY: Color32 = Color32::from_rgb(167, 139, 250);
pub const SECONDARY_MUTED: Color32 = Color32::from_rgb(45, 31, 94);

// ── Text ──────────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 232, 232);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 160);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(90, 90, 90);
pub const TEXT_ON_PRIMARY: Color32 = Color32::WHITE;

// ── Semantic ──────────────────────────────────────────────────────────────────
pub const STATUS_CONNECTED: Color32 = Color32::from_rgb(74, 222, 128);
pub const STATUS_DISCONNECTED: Color32 = Color32::from_rgb(107, 114, 128);
pub const STATUS_ERROR: Color32 = Color32::from_rgb(248, 113, 113);
pub const HIGHLIGHT_BG: Color32 = Color32::from_rgb(45, 31, 94);
pub const HIGHLIGHT_TEXT: Color32 = Color32::from_rgb(196, 181, 253);
pub const OVERLAY_SCRIM: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 140);

/// 로그 레벨에 대응하는 레이블(뱃지) 색상을 반환한다.
#[must_use]
pub fn level_label_color(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Debug => Color32::from_rgb(125, 211, 252),
        LogLevel::Info => Color32::from_rgb(74, 222, 128),
        LogLevel::Warn => Color32::from_rgb(252, 211, 77),
        LogLevel::Error => Color32::from_rgb(248, 113, 113),
        LogLevel::Fatal => Color32::from_rgb(255, 45, 111),
        LogLevel::Verbose | LogLevel::Unknown => Color32::from_rgb(107, 114, 128),
    }
}

/// 로그 레벨에 대응하는 행 배경색을 반환한다.
#[must_use]
pub fn level_row_bg(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Verbose | LogLevel::Debug | LogLevel::Unknown => Color32::from_rgb(26, 26, 26),
        LogLevel::Info => Color32::from_rgb(30, 42, 30),
        LogLevel::Warn => Color32::from_rgb(42, 36, 21),
        LogLevel::Error => Color32::from_rgb(42, 26, 26),
        LogLevel::Fatal => Color32::from_rgb(42, 15, 26),
    }
}
