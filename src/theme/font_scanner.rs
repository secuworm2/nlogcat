use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct FontInfo {
    pub display_name: String,
    pub path: String,
    pub face_index: u32,
}

static CACHE: OnceLock<Vec<FontInfo>> = OnceLock::new();

/// Returns the cached list of monospace fonts (system + bundled).
/// The first call triggers a scan; subsequent calls are instant.
pub fn system_monospace_fonts() -> &'static [FontInfo] {
    CACHE.get_or_init(scan_all)
}

// ── public warm-up ──────────────────────────────────────────────────────────

/// Call this once at startup (e.g. in a spawn_blocking task) to pre-warm the
/// cache so the settings panel opens without delay.
pub fn warm_up() {
    let _ = system_monospace_fonts();
}

// ── scanning ────────────────────────────────────────────────────────────────

fn scan_all() -> Vec<FontInfo> {
    let mut result = Vec::new();

    for dir in &[r"C:\Windows\Fonts", "assets/fonts"] {
        scan_dir(dir, &mut result);
    }

    result.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    // Deduplicate: keep first occurrence of each family name
    let mut seen = HashSet::new();
    result.retain(|f| seen.insert(f.display_name.clone()));
    result
}

fn scan_dir(dir: &str, out: &mut Vec<FontInfo>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if !matches!(ext.as_str(), "ttf" | "otf" | "ttc" | "otc") {
            continue;
        }
        let Ok(data) = std::fs::read(&path) else { continue };
        let path_str = path.to_string_lossy().into_owned();

        if data.starts_with(b"ttcf") {
            let n = read_u32(&data, 8).unwrap_or(0) as usize;
            for i in 0..n.min(64) {
                let Some(font_offset) = read_u32(&data, 12 + i * 4).map(|v| v as usize) else {
                    break;
                };
                if let Some(info) = try_parse(&data, font_offset, &path_str, i as u32) {
                    out.push(info);
                }
            }
        } else if let Some(info) = try_parse(&data, 0, &path_str, 0) {
            out.push(info);
        }
    }
}

fn try_parse(data: &[u8], font_offset: usize, path: &str, face_index: u32) -> Option<FontInfo> {
    let tables = table_directory(data, font_offset)?;
    if !is_fixed_pitch(data, &tables) {
        return None;
    }
    let display_name = family_name(data, &tables)?;
    if display_name.is_empty() {
        return None;
    }
    Some(FontInfo { display_name, path: path.to_owned(), face_index })
}

// ── TrueType binary helpers ──────────────────────────────────────────────────

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    let b = data.get(offset..offset + 2)?;
    Some(u16::from_be_bytes([b[0], b[1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let b = data.get(offset..offset + 4)?;
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

struct TableEntry {
    tag: [u8; 4],
    offset: u32,
    length: u32,
}

fn table_directory(data: &[u8], font_start: usize) -> Option<Vec<TableEntry>> {
    let num = read_u16(data, font_start + 4)? as usize;
    let mut tables = Vec::with_capacity(num);
    for i in 0..num {
        let rec = font_start + 12 + i * 16;
        let tag_bytes = data.get(rec..rec + 4)?;
        let tag = [tag_bytes[0], tag_bytes[1], tag_bytes[2], tag_bytes[3]];
        let offset = read_u32(data, rec + 8)?;
        let length = read_u32(data, rec + 12)?;
        tables.push(TableEntry { tag, offset, length });
    }
    Some(tables)
}

fn table_slice<'a>(data: &'a [u8], tables: &[TableEntry], tag: &[u8; 4]) -> Option<&'a [u8]> {
    let e = tables.iter().find(|t| &t.tag == tag)?;
    let s = e.offset as usize;
    data.get(s..s + e.length as usize)
}

/// Reads `post.isFixedPitch` (uint32 at byte offset 12 of the `post` table).
/// Nonzero means the font is monospace.
fn is_fixed_pitch(data: &[u8], tables: &[TableEntry]) -> bool {
    table_slice(data, tables, b"post")
        .and_then(|p| read_u32(p, 12))
        .map(|v| v != 0)
        .unwrap_or(false)
}

/// Reads the font family name from the `name` table.
/// Prefers name ID 16 (Preferred Family) over 1 (Font Family),
/// and Windows Unicode (platform 3, encoding 1) over other encodings.
fn family_name(data: &[u8], tables: &[TableEntry]) -> Option<String> {
    let nt = table_slice(data, tables, b"name")?;
    let count = read_u16(nt, 2)? as usize;
    let str_base = read_u16(nt, 4)? as usize;

    // (platform, encoding, lang, name_id, str_len, str_off)
    type Record = (u16, u16, u16, u16, u16, u16);
    let mut best: Option<Record> = None;

    for i in 0..count {
        let r = 6 + i * 12;
        let pid = read_u16(nt, r)?;
        let enc = read_u16(nt, r + 2)?;
        let lang = read_u16(nt, r + 4)?;
        let nid = read_u16(nt, r + 6)?;
        let len = read_u16(nt, r + 8)?;
        let off = read_u16(nt, r + 10)?;

        if nid != 1 && nid != 16 {
            continue;
        }

        let score = name_score(pid, enc, lang, nid);
        let best_score = best.map(|(p, e, l, n, _, _)| name_score(p, e, l, n)).unwrap_or(0);
        if score > best_score {
            best = Some((pid, enc, lang, nid, len, off));
        }
    }

    let (pid, _, _, _, len, off) = best?;
    let abs = str_base + off as usize;
    let bytes = nt.get(abs..abs + len as usize)?;

    if pid == 3 {
        // Windows: UTF-16 BE
        let u16s: Vec<u16> = bytes
            .chunks(2)
            .filter_map(|c| c.get(..2).map(|b| u16::from_be_bytes([b[0], b[1]])))
            .collect();
        String::from_utf16(&u16s).ok()
    } else {
        // Platform 1 (Mac) or 0 (Unicode): treat as Latin-1
        Some(bytes.iter().map(|&b| b as char).collect())
    }
}

/// Higher = preferred. Ranks by: name_id 16 > 1, Windows Unicode > other, English > other.
fn name_score(pid: u16, enc: u16, lang: u16, nid: u16) -> u32 {
    let preferred_name = if nid == 16 { 4 } else { 0 };
    let preferred_platform = if pid == 3 && enc == 1 { 2 } else { 0 };
    let preferred_lang = if lang == 0x0409 { 1 } else { 0 }; // en-US
    preferred_name + preferred_platform + preferred_lang
}
