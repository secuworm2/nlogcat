use std::ops::Range;

use egui::{text::LayoutJob, Color32, FontId, TextFormat};

// Mirror of theme::colors::{HIGHLIGHT_BG, HIGHLIGHT_TEXT} — kept here so the
// engine crate compiles without a dependency on the UI theme module.
const HIGHLIGHT_BG: Color32 = Color32::from_rgb(45, 31, 94);
const HIGHLIGHT_TEXT: Color32 = Color32::from_rgb(196, 181, 253);

pub struct SearchEngine;

impl SearchEngine {
    /// Returns byte-index ranges within `text` where `query` appears.
    /// `query_lc` is the pre-computed lowercase version of `query` (ignored when `case_sensitive`).
    #[must_use]
    pub fn highlight_ranges(
        text: &str,
        query: &str,
        query_lc: &str,
        case_sensitive: bool,
    ) -> Vec<Range<usize>> {
        if query.is_empty() || text.is_empty() {
            return Vec::new();
        }

        let mut ranges = Vec::new();

        if case_sensitive {
            let qlen = query.len();
            let mut start = 0;
            while start + qlen <= text.len() {
                if text[start..].starts_with(query) {
                    ranges.push(start..start + qlen);
                    start += qlen;
                } else {
                    start += text[start..].chars().next().map_or(1, char::len_utf8);
                }
            }
        } else {
            let text_lc = text.to_lowercase();
            let qlen = query_lc.len();
            let mut start = 0;
            while start + qlen <= text_lc.len() {
                if text_lc[start..].starts_with(query_lc) {
                    if text.is_char_boundary(start) && text.is_char_boundary(start + qlen) {
                        ranges.push(start..start + qlen);
                    }
                    start += qlen;
                } else {
                    start += text_lc[start..].chars().next().map_or(1, char::len_utf8);
                }
            }
        }

        ranges
    }

    /// Builds a `LayoutJob` that highlights the given byte ranges.
    /// Highlighted: `HIGHLIGHT_BG` background + `HIGHLIGHT_TEXT` color.
    /// Non-highlighted: `base_color`.
    #[must_use]
    pub fn build_layout_job(
        text: &str,
        ranges: &[Range<usize>],
        base_color: Color32,
        font_id: FontId,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        let mut cursor = 0usize;

        for range in ranges {
            if cursor < range.start {
                job.append(
                    &text[cursor..range.start],
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        color: base_color,
                        ..Default::default()
                    },
                );
            }
            job.append(
                &text[range.start..range.end],
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: HIGHLIGHT_TEXT,
                    background: HIGHLIGHT_BG,
                    ..Default::default()
                },
            );
            cursor = range.end;
        }

        if cursor < text.len() {
            job.append(
                &text[cursor..],
                0.0,
                TextFormat {
                    font_id,
                    color: base_color,
                    ..Default::default()
                },
            );
        }

        job
    }
}
