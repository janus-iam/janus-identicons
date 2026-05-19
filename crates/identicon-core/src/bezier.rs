use crate::blob::Point;
use smallvec::SmallVec;

const POINTS_CAPACITY: usize = 14;

/// Build a smooth closed cubic Bézier path from polar samples.
pub fn smooth_blob_path(
    cx: f32,
    cy: f32,
    base_radius: f32,
    point_count: usize,
    curve_strength: f32,
    perturbations: &[f32],
) -> SmallVec<[Point; POINTS_CAPACITY]> {
    let n = point_count.clamp(6, POINTS_CAPACITY);
    let mut pts: SmallVec<[Point; POINTS_CAPACITY]> = SmallVec::new();

    for i in 0..n {
        let theta = (i as f32 / n as f32) * std::f32::consts::TAU;
        let perturb = perturbations.get(i).copied().unwrap_or(0.0);
        let r = base_radius * (1.0 + curve_strength * perturb);
        pts.push(Point {
            x: cx + theta.cos() * r,
            y: cy + theta.sin() * r,
        });
    }
    pts
}

/// Write SVG path `d` for a closed smooth curve through points.
pub fn write_path_d(out: &mut String, points: &[Point]) {
    let n = points.len();
    if n < 3 {
        return;
    }

    out.push('M');
    write_pair(out, points[0].x, points[0].y);

    for i in 0..n {
        let p0 = &points[(i + n - 1) % n];
        let p1 = &points[i];
        let p2 = &points[(i + 1) % n];
        let p3 = &points[(i + 2) % n];

        let cp1x = p1.x + (p2.x - p0.x) / 6.0;
        let cp1y = p1.y + (p2.y - p0.y) / 6.0;
        let cp2x = p2.x - (p3.x - p1.x) / 6.0;
        let cp2y = p2.y - (p3.y - p1.y) / 6.0;

        out.push(' ');
        out.push('C');
        write_pair(out, cp1x, cp1y);
        write_pair(out, cp2x, cp2y);
        write_pair(out, p2.x, p2.y);
    }
    out.push('Z');
}

fn write_pair(out: &mut String, x: f32, y: f32) {
    write_f32(out, x);
    out.push(',');
    write_f32(out, y);
}

pub fn write_f32(out: &mut String, v: f32) {
    let rounded = (v * 10.0).round() / 10.0;
    if rounded.fract().abs() < 0.05 {
        let i = rounded as i32;
        out.push_str(&i.to_string());
    } else {
        out.push_str(&format!("{rounded:.1}"));
    }
}
