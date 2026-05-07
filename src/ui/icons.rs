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

#[allow(clippy::cast_precision_loss)]
pub fn refresh(painter: &Painter, rect: Rect, color: Color32) {
    use std::f32::consts::PI;
    let cx = rect.center().x;
    let cy = rect.center().y;
    let r = rect.width().min(rect.height()) * 0.36;
    let stroke = Stroke::new(1.5, color);

    let start = -PI / 2.0 + PI / 6.0;
    let end = -PI / 2.0 + 2.0 * PI - PI / 6.0;
    let arc: Vec<Pos2> = (0..=32)
        .map(|i| {
            let a = start + (end - start) * i as f32 / 32.0;
            Pos2::new(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect();
    painter.add(Shape::line(arc, stroke));

    let tip = Pos2::new(cx + r * end.cos(), cy + r * end.sin());
    let tx = -end.sin();
    let ty = end.cos();
    let al = r * 0.58;
    let aw = r * 0.34;
    let base = Pos2::new(tip.x - tx * al, tip.y - ty * al);
    let a1 = Pos2::new(base.x - ty * aw, base.y + tx * aw);
    let a2 = Pos2::new(base.x + ty * aw, base.y - tx * aw);
    painter.add(Shape::convex_polygon(vec![tip, a1, a2], color, Stroke::NONE));
}

#[allow(clippy::cast_precision_loss)]
pub fn gear(painter: &Painter, rect: Rect, color: Color32) {
    use std::f32::consts::PI;
    let cx = rect.center().x;
    let cy = rect.center().y;
    let outer_r = rect.width().min(rect.height()) * 0.44;
    let inner_r = outer_r * 0.68;
    let hole_r = outer_r * 0.28;
    let n: usize = 8;
    let period = PI * 2.0 / n as f32;
    let tooth_half = period * 0.25;
    let s = Stroke::new(1.5, color);

    let mut pts: Vec<Pos2> = Vec::with_capacity(3 * n);
    for i in 0..n {
        let ca = period * i as f32;
        for &(r, da) in &[(inner_r, -period * 0.5), (outer_r, -tooth_half), (outer_r, tooth_half)] {
            let a = ca + da;
            pts.push(Pos2::new(cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    painter.add(Shape::closed_line(pts, s));
    painter.circle_stroke(Pos2::new(cx, cy), hole_r, s);
}
