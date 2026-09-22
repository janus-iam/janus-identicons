use crate::bezier::write_f32;
use crate::palette::Palette;
use std::f32::consts::TAU;

/// Shared drawing context for the non-blob engines.
///
/// All element ids are suffixed with a short hash-derived token so that several
/// identicons can be inlined in the same HTML document without gradient/clip
/// collisions.
pub struct Ctx<'a> {
    pub size: f32,
    pub palette: &'a Palette,
    pub hash: &'a [u8; 32],
    pub animated: bool,
    pub background: bool,
    uid: String,
    out: String,
}

impl<'a> Ctx<'a> {
    pub fn new(
        size: u32,
        palette: &'a Palette,
        hash: &'a [u8; 32],
        animated: bool,
        background: bool,
    ) -> Self {
        let mut uid = String::with_capacity(6);
        for b in &hash[28..31] {
            uid.push_str(&format!("{b:02x}"));
        }
        Self {
            size: size as f32,
            palette,
            hash,
            animated,
            background,
            uid,
            out: String::with_capacity(6144),
        }
    }

    pub fn into_string(self) -> String {
        self.out
    }

    pub fn out(&mut self) -> &mut String {
        &mut self.out
    }

    pub fn center(&self) -> f32 {
        self.size * 0.5
    }

    /// Hash byte `i` reduced modulo `n`; used to allocate dedicated bits to
    /// macroscopic traits so that close inputs diverge on the big features.
    pub fn trait_u8(&self, i: usize, n: u8) -> u8 {
        self.hash[i % 32] % n
    }

    pub fn id(&self, name: &str) -> String {
        let mut s = String::with_capacity(name.len() + 7);
        s.push_str(name);
        s.push('-');
        s.push_str(&self.uid);
        s
    }

    pub fn color(&self, index: usize) -> &'static str {
        self.palette.colors[index % self.palette.colors.len()]
    }

    /// Three distinct palette indices chosen from a seed (60 ordered triples).
    pub fn distinct3(seed: u32) -> [usize; 3] {
        let s = seed as usize;
        let a = s % 5;
        let mut b = (s / 5) % 4;
        if b >= a {
            b += 1;
        }
        let mut c = (s / 20) % 3;
        for taken in [a.min(b), a.max(b)] {
            if c >= taken {
                c += 1;
            }
        }
        [a, b, c]
    }

    pub fn begin(&mut self) {
        let s = self.size as u32;
        self.out.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {s} {s}" width="{s}" height="{s}" role="img" aria-hidden="true"><defs>"#
        ));
        if self.background {
            let bg = self.id("bg");
            let accent = self.color(self.trait_u8(30, 5) as usize);
            self.out.push_str(&format!(
                r#"<radialGradient id="{bg}" cx="0.5" cy="0.4" r="0.8"><stop offset="0" stop-color="{}" stop-opacity="1"/><stop offset="1" stop-color="{accent}" stop-opacity="0.16"/></radialGradient>"#,
                self.palette.bg_dark
            ));
        }
    }

    pub fn close_defs(&mut self) {
        self.out.push_str("</defs>");
        if self.background {
            let bg = self.id("bg");
            self.out.push_str(&format!(
                r#"<rect width="100%" height="100%" fill="{}"/><rect width="100%" height="100%" fill="url(#{bg})"/>"#,
                self.palette.bg_dark
            ));
        }
    }

    pub fn finish(&mut self) {
        self.out.push_str("</svg>");
    }

    pub fn write_circle(&mut self, cx: f32, cy: f32, r: f32, fill: &str, opacity: f32) {
        self.out.push_str("<circle cx=\"");
        write_f32(&mut self.out, cx);
        self.out.push_str("\" cy=\"");
        write_f32(&mut self.out, cy);
        self.out.push_str("\" r=\"");
        write_f32(&mut self.out, r);
        self.out.push_str("\" fill=\"");
        self.out.push_str(fill);
        if opacity < 1.0 {
            self.out.push_str("\" fill-opacity=\"");
            write_f32(&mut self.out, opacity);
        }
        self.out.push_str("\"/>");
    }

    pub fn write_ring(&mut self, cx: f32, cy: f32, r: f32, stroke: &str, width: f32, opacity: f32) {
        self.out.push_str("<circle cx=\"");
        write_f32(&mut self.out, cx);
        self.out.push_str("\" cy=\"");
        write_f32(&mut self.out, cy);
        self.out.push_str("\" r=\"");
        write_f32(&mut self.out, r);
        self.out.push_str("\" fill=\"none\" stroke=\"");
        self.out.push_str(stroke);
        self.out.push_str("\" stroke-width=\"");
        write_f32(&mut self.out, width);
        self.out.push_str("\" stroke-opacity=\"");
        write_f32(&mut self.out, opacity);
        self.out.push_str("\"/>");
    }

    pub fn write_line(
        &mut self,
        a: (f32, f32),
        b: (f32, f32),
        stroke: &str,
        width: f32,
        opacity: f32,
    ) {
        self.out.push_str("<line x1=\"");
        write_f32(&mut self.out, a.0);
        self.out.push_str("\" y1=\"");
        write_f32(&mut self.out, a.1);
        self.out.push_str("\" x2=\"");
        write_f32(&mut self.out, b.0);
        self.out.push_str("\" y2=\"");
        write_f32(&mut self.out, b.1);
        self.out.push_str("\" stroke=\"");
        self.out.push_str(stroke);
        self.out.push_str("\" stroke-width=\"");
        write_f32(&mut self.out, width);
        self.out.push_str("\" stroke-opacity=\"");
        write_f32(&mut self.out, opacity);
        self.out.push_str("\" stroke-linecap=\"round\"/>");
    }

    /// Writes `<path d="..." attrs/>` where `attrs` is raw attribute text.
    pub fn write_path(&mut self, d: &str, attrs: &str) {
        self.out.push_str("<path d=\"");
        self.out.push_str(d);
        self.out.push_str("\" ");
        self.out.push_str(attrs);
        self.out.push_str("/>");
    }

    pub fn write_polygon(&mut self, pts: &[(f32, f32)], attrs: &str) {
        let mut d = String::with_capacity(pts.len() * 12);
        polygon_d(&mut d, pts);
        self.write_path(&d, attrs);
    }
}

pub fn polar(cx: f32, cy: f32, r: f32, angle: f32) -> (f32, f32) {
    (cx + r * angle.cos(), cy + r * angle.sin())
}

pub fn write_pt(out: &mut String, p: (f32, f32)) {
    write_f32(out, p.0);
    out.push(',');
    write_f32(out, p.1);
}

pub fn polygon_d(out: &mut String, pts: &[(f32, f32)]) {
    for (i, p) in pts.iter().enumerate() {
        out.push(if i == 0 { 'M' } else { 'L' });
        write_pt(out, *p);
    }
    out.push('Z');
}

/// Closed Catmull-Rom spline through `pts`, written as cubic Béziers with
/// explicit separators between every coordinate pair.
pub fn smooth_closed_d(out: &mut String, pts: &[(f32, f32)]) {
    let n = pts.len();
    if n < 3 {
        return;
    }
    out.push('M');
    write_pt(out, pts[0]);
    for i in 0..n {
        let p0 = pts[(i + n - 1) % n];
        let p1 = pts[i];
        let p2 = pts[(i + 1) % n];
        let p3 = pts[(i + 2) % n];
        let cp1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let cp2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        out.push('C');
        write_pt(out, cp1);
        out.push(' ');
        write_pt(out, cp2);
        out.push(' ');
        write_pt(out, p2);
    }
    out.push('Z');
}

pub fn regular_polygon(cx: f32, cy: f32, r: f32, n: usize, rotation: f32) -> Vec<(f32, f32)> {
    (0..n)
        .map(|i| polar(cx, cy, r, rotation + TAU * i as f32 / n as f32))
        .collect()
}

pub fn star_polygon(
    cx: f32,
    cy: f32,
    r_out: f32,
    r_in: f32,
    n: usize,
    rotation: f32,
) -> Vec<(f32, f32)> {
    (0..n * 2)
        .map(|i| {
            let r = if i % 2 == 0 { r_out } else { r_in };
            polar(cx, cy, r, rotation + TAU * i as f32 / (n * 2) as f32)
        })
        .collect()
}

/// Outer silhouette of an identicon: the single most recognisable trait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contour {
    Circle,
    RoundedSquare,
    Hexagon,
    Shield,
    Diamond,
}

impl Contour {
    pub fn from_index(i: u8) -> Self {
        match i % 5 {
            0 => Contour::Circle,
            1 => Contour::RoundedSquare,
            2 => Contour::Hexagon,
            3 => Contour::Shield,
            _ => Contour::Diamond,
        }
    }

    /// Path `d` for the contour centred at (cx, cy) with half-extent `r`.
    pub fn path_d(self, cx: f32, cy: f32, r: f32) -> String {
        let mut d = String::with_capacity(160);
        match self {
            Contour::Circle => {
                d.push('M');
                write_pt(&mut d, (cx - r, cy));
                d.push('a');
                write_pt(&mut d, (r, r));
                d.push_str(" 0 1 0 ");
                write_pt(&mut d, (2.0 * r, 0.0));
                d.push('a');
                write_pt(&mut d, (r, r));
                d.push_str(" 0 1 0 ");
                write_pt(&mut d, (-2.0 * r, 0.0));
                d.push('Z');
            }
            Contour::RoundedSquare => {
                let k = r * 0.28;
                let (x0, y0, x1, y1) = (cx - r, cy - r, cx + r, cy + r);
                d.push('M');
                write_pt(&mut d, (x0 + k, y0));
                d.push('L');
                write_pt(&mut d, (x1 - k, y0));
                d.push('Q');
                write_pt(&mut d, (x1, y0));
                d.push(' ');
                write_pt(&mut d, (x1, y0 + k));
                d.push('L');
                write_pt(&mut d, (x1, y1 - k));
                d.push('Q');
                write_pt(&mut d, (x1, y1));
                d.push(' ');
                write_pt(&mut d, (x1 - k, y1));
                d.push('L');
                write_pt(&mut d, (x0 + k, y1));
                d.push('Q');
                write_pt(&mut d, (x0, y1));
                d.push(' ');
                write_pt(&mut d, (x0, y1 - k));
                d.push('L');
                write_pt(&mut d, (x0, y0 + k));
                d.push('Q');
                write_pt(&mut d, (x0, y0));
                d.push(' ');
                write_pt(&mut d, (x0 + k, y0));
                d.push('Z');
            }
            Contour::Hexagon => {
                let pts = regular_polygon(cx, cy, r * 1.04, 6, -TAU / 4.0);
                polygon_d(&mut d, &pts);
            }
            Contour::Diamond => {
                let pts = regular_polygon(cx, cy, r * 1.08, 4, -TAU / 4.0);
                polygon_d(&mut d, &pts);
            }
            Contour::Shield => {
                let w = r * 0.92;
                let top = cy - r * 0.95;
                let mid = cy + r * 0.15;
                let bottom = cy + r * 1.02;
                d.push('M');
                write_pt(&mut d, (cx - w, top));
                d.push('L');
                write_pt(&mut d, (cx + w, top));
                d.push('L');
                write_pt(&mut d, (cx + w, mid));
                d.push('C');
                write_pt(&mut d, (cx + w, mid + r * 0.55));
                d.push(' ');
                write_pt(&mut d, (cx + r * 0.35, bottom - r * 0.12));
                d.push(' ');
                write_pt(&mut d, (cx, bottom));
                d.push('C');
                write_pt(&mut d, (cx - r * 0.35, bottom - r * 0.12));
                d.push(' ');
                write_pt(&mut d, (cx - w, mid + r * 0.55));
                d.push(' ');
                write_pt(&mut d, (cx - w, mid));
                d.push('Z');
            }
        }
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct3_are_distinct_and_in_range() {
        for seed in 0..300u32 {
            let [a, b, c] = Ctx::distinct3(seed);
            assert!(a < 5 && b < 5 && c < 5, "seed {seed}");
            assert!(a != b && b != c && a != c, "seed {seed}: {a} {b} {c}");
        }
    }

    #[test]
    fn contour_paths_are_closed() {
        for i in 0..5 {
            let d = Contour::from_index(i).path_d(50.0, 50.0, 40.0);
            assert!(d.starts_with('M') && d.ends_with('Z'));
        }
    }
}
