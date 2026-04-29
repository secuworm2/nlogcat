use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke};

fn p(rect: &Rect, fx: f32, fy: f32) -> Pos2 {
    Pos2::new(rect.min.x + rect.width() * fx, rect.min.y + rect.height() * fy)
}

pub fn play(painter: &Painter, rect: Rect, color: Color32) {
    painter.add(Shape::convex_polygon(
        vec![p(&rect, 0.18, 0.1), p(&rect, 0.18, 0.9), p(&rect, 0.88, 0.5)],
        color,
        Stroke::NONE,
    ));
}

pub fn pause(painter: &Painter, rect: Rect, color: Color32) {
    let bw = rect.width() * 0.28;
    let py = rect.height() * 0.1;
    for &lx in &[rect.min.x + rect.width() * 0.12, rect.min.x + rect.width() * 0.58] {
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(lx, rect.min.y + py),
                egui::vec2(bw, rect.height() - 2.0 * py),
            ),
            1.5,
            color,
        );
    }
}

pub fn trash(painter: &Painter, rect: Rect, color: Color32) {
    let s = Stroke::new(1.5, color);
    // Handle tab
    painter.rect_filled(
        Rect::from_min_max(p(&rect, 0.35, 0.0), p(&rect, 0.65, 0.2)),
        1.5,
        color,
    );
    // Lid
    painter.hline(
        (rect.min.x + rect.width() * 0.08)..=(rect.max.x - rect.width() * 0.08),
        rect.min.y + rect.height() * 0.27,
        s,
    );
    // Body
    painter.rect_stroke(
        Rect::from_min_max(p(&rect, 0.17, 0.34), p(&rect, 0.83, 0.94)),
        2.0,
        s,
    );
}

pub fn save(painter: &Painter, rect: Rect, color: Color32) {
    let s = Stroke::new(1.5, color);
    let cx = rect.center().x;
    // Arrow shaft
    painter.vline(
        cx,
        (rect.min.y + rect.height() * 0.08)..=(rect.min.y + rect.height() * 0.56),
        s,
    );
    // Arrow head (filled triangle)
    painter.add(Shape::convex_polygon(
        vec![p(&rect, 0.5, 0.78), p(&rect, 0.2, 0.5), p(&rect, 0.8, 0.5)],
        color,
        Stroke::NONE,
    ));
    // Base line
    painter.hline(
        (rect.min.x + rect.width() * 0.12)..=(rect.max.x - rect.width() * 0.12),
        rect.max.y - rect.height() * 0.1,
        s,
    );
}

/// Sliders icon (modern settings metaphor): 3 horizontal bars with circular knobs.
pub fn settings(painter: &Painter, rect: Rect, color: Color32) {
    let s = Stroke::new(1.5, color);
    let pad_x = rect.width() * 0.06;
    let knob_r = rect.height() * 0.13;

    for (fy, kfx) in [(0.25_f32, 0.70_f32), (0.5, 0.38), (0.75, 0.62)] {
        let y = rect.min.y + rect.height() * fy;
        let kx = rect.min.x + rect.width() * kfx;
        let knob = Pos2::new(kx, y);

        // Bar left of knob
        if kx - knob_r - 1.0 > rect.min.x + pad_x {
            painter.hline((rect.min.x + pad_x)..=(kx - knob_r - 1.0), y, s);
        }
        // Bar right of knob
        if kx + knob_r + 1.0 < rect.max.x - pad_x {
            painter.hline((kx + knob_r + 1.0)..=(rect.max.x - pad_x), y, s);
        }
        // Knob outline only
        painter.circle_stroke(knob, knob_r, s);
    }
}
