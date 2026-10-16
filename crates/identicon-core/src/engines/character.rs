//! Layered character identicon.
//!
//! Every character shares one face: the same round head, the same eyes and
//! the same smile. A hash only changes the outfit (hat, glasses, neck
//! accessory) and the setting behind them. Geometry is original SVG — no
//! filters, no images.

use super::common::{Ctx, polygon_d, star_polygon};
use crate::bezier::write_f32;
use crate::prng::Prng;
use std::f32::consts::FRAC_PI_2;

const INK: &str = "#1c1917";
const PAPER: &str = "#f6f1e7";
const BLUSH: &str = "#ff6d8a";
/// One light face colour so the eyes stay the same ink on every character.
const FACE: &str = "#8fd0ef";
const LENS: &str = "#121820";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hat {
    None,
    Beanie,
    Top,
    Cap,
    Beret,
    Crown,
    Party,
    Bow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Glasses {
    None,
    Round,
    Square,
    Sun,
    Cat,
    Monocle,
    Aviator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Neck {
    None,
    Scarf,
    Bowtie,
    Necklace,
    Collar,
    Tie,
    Bandana,
    Choker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Backdrop {
    Sky,
    Mountain,
    Sea,
    Forest,
    Sunset,
    Beach,
    Night,
    Snow,
}

struct Metrics {
    crown: f32,
    neck: f32,
    half: f32,
    eye_y: f32,
    mouth_y: f32,
    eye_dx: f32,
}

struct Spec {
    hat: Hat,
    glasses: Glasses,
    neck: Neck,
    bg: Option<Backdrop>,
    flip: u8,
}

struct Face {
    lx: f32,
    rx: f32,
    ey: f32,
    mx: f32,
    my: f32,
    ex: f32,
    ey_r: f32,
    lens: f32,
}

struct Colors {
    body: &'static str,
    accent: &'static str,
    jewel: &'static str,
    fg: &'static str,
}

struct Path {
    d: String,
    k: f32,
}

impl Hat {
    const ALL: [Hat; 8] = [
        Hat::None,
        Hat::Beanie,
        Hat::Top,
        Hat::Cap,
        Hat::Beret,
        Hat::Crown,
        Hat::Party,
        Hat::Bow,
    ];

    fn from_u8(v: u8) -> Self {
        Self::ALL[(v % 8) as usize]
    }

    fn name(self) -> &'static str {
        match self {
            Hat::None => "none",
            Hat::Beanie => "beanie",
            Hat::Top => "top",
            Hat::Cap => "cap",
            Hat::Beret => "beret",
            Hat::Crown => "crown",
            Hat::Party => "party",
            Hat::Bow => "bow",
        }
    }
}

impl Glasses {
    const ALL: [Glasses; 7] = [
        Glasses::None,
        Glasses::Round,
        Glasses::Square,
        Glasses::Sun,
        Glasses::Cat,
        Glasses::Monocle,
        Glasses::Aviator,
    ];

    fn from_u8(v: u8) -> Self {
        Self::ALL[(v % 7) as usize]
    }

    fn name(self) -> &'static str {
        match self {
            Glasses::None => "none",
            Glasses::Round => "round",
            Glasses::Square => "square",
            Glasses::Sun => "sun",
            Glasses::Cat => "cat",
            Glasses::Monocle => "monocle",
            Glasses::Aviator => "aviator",
        }
    }
}

impl Neck {
    const ALL: [Neck; 8] = [
        Neck::None,
        Neck::Scarf,
        Neck::Bowtie,
        Neck::Necklace,
        Neck::Collar,
        Neck::Tie,
        Neck::Bandana,
        Neck::Choker,
    ];

    fn from_u8(v: u8) -> Self {
        Self::ALL[(v % 8) as usize]
    }

    fn name(self) -> &'static str {
        match self {
            Neck::None => "none",
            Neck::Scarf => "scarf",
            Neck::Bowtie => "bowtie",
            Neck::Necklace => "necklace",
            Neck::Collar => "collar",
            Neck::Tie => "tie",
            Neck::Bandana => "bandana",
            Neck::Choker => "choker",
        }
    }
}

impl Backdrop {
    const ALL: [Backdrop; 8] = [
        Backdrop::Sky,
        Backdrop::Mountain,
        Backdrop::Sea,
        Backdrop::Forest,
        Backdrop::Sunset,
        Backdrop::Beach,
        Backdrop::Night,
        Backdrop::Snow,
    ];

    fn from_u8(v: u8) -> Self {
        Self::ALL[(v % 8) as usize]
    }

    fn name(self) -> &'static str {
        match self {
            Backdrop::Sky => "sky",
            Backdrop::Mountain => "mountain",
            Backdrop::Sea => "sea",
            Backdrop::Forest => "forest",
            Backdrop::Sunset => "sunset",
            Backdrop::Beach => "beach",
            Backdrop::Night => "night",
            Backdrop::Snow => "snow",
        }
    }
}

impl Spec {
    fn from_hash(hash: &[u8; 32], background: bool) -> Self {
        Self {
            hat: Hat::from_u8(hash[1]),
            glasses: Glasses::from_u8(hash[2]),
            neck: Neck::from_u8(hash[3]),
            bg: background.then(|| Backdrop::from_u8(hash[4])),
            flip: hash[8],
        }
    }

    fn cap_right(&self) -> bool {
        self.flip & 1 == 0
    }

    fn monocle_left(&self) -> bool {
        self.flip & 2 == 0
    }

    fn pendant(&self) -> u8 {
        (self.flip >> 2) % 3
    }
}

/// One round head. The crown sits just under the top of the circle so a
/// band of scenery stays visible above the hat.
const FACE_METRICS: Metrics = Metrics {
    crown: 33.0,
    neck: 78.0,
    half: 16.0,
    eye_y: 52.0,
    mouth_y: 66.0,
    eye_dx: 11.2,
};

fn face_of(m: &Metrics) -> Face {
    let lens = (m.eye_dx - 2.4).clamp(6.2, 8.6);
    Face {
        lx: 50.0 - m.eye_dx,
        rx: 50.0 + m.eye_dx,
        ey: m.eye_y,
        mx: 50.0,
        my: m.mouth_y,
        ex: 4.5,
        ey_r: 6.4,
        lens,
    }
}

fn colors(ctx: &Ctx<'_>) -> Colors {
    let seed = u32::from(ctx.hash[6]) | (u32::from(ctx.hash[7]) << 8);
    let [_skip, accent_i, jewel_i] = Ctx::distinct3(seed);
    Colors {
        body: FACE,
        accent: ctx.color(accent_i),
        jewel: ctx.color(jewel_i),
        fg: INK,
    }
}

impl Path {
    fn new(k: f32) -> Self {
        Self {
            d: String::with_capacity(160),
            k,
        }
    }

    fn pt(&mut self, x: f32, y: f32) {
        write_f32(&mut self.d, x * self.k);
        self.d.push(',');
        write_f32(&mut self.d, y * self.k);
    }

    fn m(&mut self, x: f32, y: f32) {
        self.d.push('M');
        self.pt(x, y);
    }

    fn l(&mut self, x: f32, y: f32) {
        self.d.push('L');
        self.pt(x, y);
    }

    fn q(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.d.push('Q');
        self.pt(cx, cy);
        self.d.push(' ');
        self.pt(x, y);
    }

    fn c(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.d.push('C');
        self.pt(x1, y1);
        self.d.push(' ');
        self.pt(x2, y2);
        self.d.push(' ');
        self.pt(x, y);
    }

    fn z(&mut self) {
        self.d.push('Z');
    }
}

fn n(v: f32) -> String {
    let mut s = String::with_capacity(8);
    write_f32(&mut s, v);
    s
}

fn rise(m: &Metrics, factor: f32) -> f32 {
    (m.half * factor).min(m.crown - 9.0).max(8.0)
}

pub fn render(ctx: &mut Ctx<'_>, _prng: &mut Prng) {
    let spec = Spec::from_hash(ctx.hash, ctx.background);
    paint(ctx, &spec);
}

fn paint(ctx: &mut Ctx<'_>, spec: &Spec) {
    let k = ctx.size / 100.0;
    let cols = colors(ctx);
    let m = FACE_METRICS;
    let face = face_of(&m);
    let clip = ctx.id("body");
    let sheen = ctx.id("sheen");

    ctx.begin();
    {
        let out = ctx.out();
        out.push_str(&format!(r#"<clipPath id="{clip}">"#));
        write_head(out, k, "");
        out.push_str("</clipPath>");
        out.push_str(&format!(
            r##"<radialGradient id="{sheen}" cx="0.35" cy="0.28" r="0.8"><stop offset="0" stop-color="#ffffff" stop-opacity="0.5"/><stop offset="0.48" stop-color="#ffffff" stop-opacity="0"/><stop offset="1" stop-color="#000000" stop-opacity="0.22"/></radialGradient>"##
        ));
    }
    ctx.close_defs();

    ctx.out().push_str(&format!(
        r#"<g data-shape="round" data-hat="{}" data-glasses="{}" data-neck="{}" data-bg="{}" data-mood="smile">"#,
        spec.hat.name(),
        spec.glasses.name(),
        spec.neck.name(),
        spec.bg.map(Backdrop::name).unwrap_or("none"),
    ));

    if let Some(bg) = spec.bg {
        draw_scene(ctx, k, bg);
    }

    let bob = n(ctx.size * 0.014);
    let dur = format_dur(2.7 + f32::from(ctx.hash[26] % 14) * 0.15);
    ctx.out().push_str("<g>");
    if ctx.animated {
        ctx.out().push_str(&format!(
            r#"<animateTransform attributeName="transform" type="translate" values="0 0;0 -{bob};0 0" dur="{dur}s" repeatCount="indefinite"/>"#
        ));
    }

    ctx.out().push_str(&format!(
        r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#000" fill-opacity="0.16"/>"##,
        n(50.0 * k),
        n(88.0 * k),
        n(16.0 * k),
        n(3.2 * k)
    ));

    let body_attrs = format!(
        r##"fill="{}" stroke="#000000" stroke-opacity="0.16" stroke-width="{}""##,
        cols.body,
        n(k * 1.15)
    );
    write_head(ctx.out(), k, &body_attrs);

    let side = n(ctx.size);
    ctx.out()
        .push_str(&format!(r#"<g clip-path="url(#{clip})">"#));
    ctx.out().push_str(&format!(
        r#"<rect x="0" y="0" width="{side}" height="{side}" fill="url(#{sheen})"/>"#
    ));
    draw_face(ctx, k, &face);
    ctx.out().push_str("</g>");

    draw_neck(ctx, spec, k, &m, &cols);
    draw_glasses(ctx, spec, k, &face, &cols);
    draw_hat(ctx, spec, k, &m, &cols);

    ctx.out().push_str("</g></g>");
}

fn format_dur(v: f32) -> String {
    let tenths = (v * 10.0).round() as i32;
    format!("{}.{}", tenths / 10, tenths.abs() % 10)
}

fn write_head(out: &mut String, k: f32, extra: &str) {
    out.push_str(&format!(
        r#"<circle cx="{}" cy="{}" r="{}" {extra}/>"#,
        n(50.0 * k),
        n(58.0 * k),
        n(27.0 * k)
    ));
}

fn fill_rect(ctx: &mut Ctx<'_>, k: f32, x: f32, y: f32, w: f32, h: f32, fill: &str) {
    ctx.out().push_str(&format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{fill}"/>"#,
        n(x * k),
        n(y * k),
        n(w * k),
        n(h * k)
    ));
}

fn poly(ctx: &mut Ctx<'_>, k: f32, pts: &[(f32, f32)], fill: &str) {
    let mut path = Path::new(k);
    let (x, y) = pts[0];
    path.m(x, y);
    for &(px, py) in &pts[1..] {
        path.l(px, py);
    }
    path.z();
    ctx.write_path(&path.d, &format!(r#"fill="{fill}""#));
}

fn wave(ctx: &mut Ctx<'_>, k: f32, y: f32, amp: f32, color: &str) {
    let mut path = Path::new(k);
    path.m(0.0, y);
    path.q(12.0, y - amp, 25.0, y);
    path.q(38.0, y + amp, 50.0, y);
    path.q(62.0, y - amp, 75.0, y);
    path.q(88.0, y + amp, 100.0, y);
    ctx.out().push_str(&format!(
        r#"<path d="{}" fill="none" stroke="{color}" stroke-width="{}" stroke-linecap="round"/>"#,
        path.d,
        n(1.7 * k)
    ));
}

fn pine(ctx: &mut Ctx<'_>, k: f32, x: f32, tip: f32) {
    poly(
        ctx,
        k,
        &[(x, tip), (x - 7.5, tip + 16.0), (x + 7.5, tip + 16.0)],
        "#1e6b38",
    );
    poly(
        ctx,
        k,
        &[
            (x, tip + 10.0),
            (x - 11.0, tip + 30.0),
            (x + 11.0, tip + 30.0),
        ],
        "#164e2a",
    );
    fill_rect(ctx, k, x - 1.5, tip + 28.0, 3.0, 9.0, "#6b4423");
}

fn cloud(ctx: &mut Ctx<'_>, k: f32, x: f32, y: f32) {
    ctx.out().push_str(&format!(
        r##"<g fill="#ffffff"><ellipse cx="{}" cy="{}" rx="{}" ry="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}"/></g>"##,
        n((x - 6.0) * k),
        n((y + 1.5) * k),
        n(7.0 * k),
        n(4.2 * k),
        n(x * k),
        n(y * k),
        n(9.0 * k),
        n(5.4 * k),
        n((x + 7.0) * k),
        n((y + 1.2) * k),
        n(6.2 * k),
        n(3.8 * k)
    ));
}

fn draw_scene(ctx: &mut Ctx<'_>, k: f32, bg: Backdrop) {
    match bg {
        Backdrop::Sky => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 100.0, "#7ec8f8");
            ctx.write_circle(84.0 * k, 16.0 * k, 8.0 * k, "#ffe566", 1.0);
            cloud(ctx, k, 18.0, 18.0);
            cloud(ctx, k, 14.0, 72.0);
        }
        Backdrop::Mountain => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 100.0, "#8ecaf6");
            ctx.write_circle(84.0 * k, 14.0 * k, 7.0 * k, "#ffe566", 1.0);
            poly(
                ctx,
                k,
                &[
                    (0.0, 82.0),
                    (18.0, 52.0),
                    (36.0, 68.0),
                    (54.0, 48.0),
                    (72.0, 64.0),
                    (88.0, 46.0),
                    (100.0, 62.0),
                    (100.0, 100.0),
                    (0.0, 100.0),
                ],
                "#9aa8b8",
            );
            poly(
                ctx,
                k,
                &[(0.0, 100.0), (0.0, 72.0), (14.0, 46.0), (30.0, 100.0)],
                "#4f7d52",
            );
            poly(
                ctx,
                k,
                &[(70.0, 100.0), (86.0, 42.0), (100.0, 64.0), (100.0, 100.0)],
                "#3e6844",
            );
            poly(
                ctx,
                k,
                &[(10.5, 54.0), (14.0, 46.0), (17.5, 54.0)],
                "#f7fbff",
            );
            poly(
                ctx,
                k,
                &[(82.5, 50.0), (86.0, 42.0), (89.5, 50.0)],
                "#f7fbff",
            );
        }
        Backdrop::Sea => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 46.0, "#7ec8f8");
            fill_rect(ctx, k, 0.0, 44.0, 100.0, 56.0, "#1277b8");
            ctx.write_circle(16.0 * k, 44.0 * k, 8.0 * k, "#ffe566", 1.0);
            wave(ctx, k, 58.0, 3.2, "#d7f4ff");
            wave(ctx, k, 72.0, 2.6, "#7fd4f0");
            wave(ctx, k, 86.0, 2.2, "#d7f4ff");
        }
        Backdrop::Forest => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 100.0, "#b7e3fb");
            fill_rect(ctx, k, 0.0, 74.0, 100.0, 26.0, "#3e9a45");
            ctx.write_circle(82.0 * k, 16.0 * k, 6.5 * k, "#ffe566", 1.0);
            pine(ctx, k, 12.0, 22.0);
            pine(ctx, k, 88.0, 28.0);
            cloud(ctx, k, 48.0, 12.0);
        }
        Backdrop::Sunset => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 34.0, "#ffb15e");
            fill_rect(ctx, k, 0.0, 34.0, 100.0, 24.0, "#ff6d4a");
            fill_rect(ctx, k, 0.0, 58.0, 100.0, 42.0, "#3a2218");
            ctx.write_circle(14.0 * k, 50.0 * k, 9.0 * k, "#ffe9a0", 1.0);
            poly(
                ctx,
                k,
                &[
                    (0.0, 100.0),
                    (0.0, 70.0),
                    (18.0, 62.0),
                    (34.0, 74.0),
                    (66.0, 64.0),
                    (84.0, 76.0),
                    (100.0, 66.0),
                    (100.0, 100.0),
                ],
                "#24140f",
            );
        }
        Backdrop::Beach => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 42.0, "#7ec8f8");
            ctx.write_circle(84.0 * k, 16.0 * k, 7.0 * k, "#ffe566", 1.0);
            fill_rect(ctx, k, 0.0, 38.0, 100.0, 30.0, "#2f9fd0");
            fill_rect(ctx, k, 0.0, 66.0, 100.0, 34.0, "#f2d2a0");
            wave(ctx, k, 48.0, 2.4, "#d7f4ff");
            wave(ctx, k, 58.0, 2.0, "#9fe3f5");
        }
        Backdrop::Night => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 100.0, "#101a38");
            ctx.write_circle(16.0 * k, 18.0 * k, 8.0 * k, "#f4efd0", 1.0);
            ctx.write_circle(19.4 * k, 15.6 * k, 6.4 * k, "#101a38", 1.0);
            for (x, y, r) in [
                (74.0, 12.0, 1.1),
                (86.0, 20.0, 0.8),
                (92.0, 10.0, 1.3),
                (78.0, 32.0, 0.7),
                (68.0, 18.0, 0.6),
                (88.0, 42.0, 0.9),
            ] {
                ctx.write_circle(x * k, y * k, r * k, "#fff6c2", 1.0);
            }
            poly(
                ctx,
                k,
                &[
                    (0.0, 100.0),
                    (0.0, 78.0),
                    (22.0, 86.0),
                    (40.0, 74.0),
                    (62.0, 88.0),
                    (82.0, 72.0),
                    (100.0, 82.0),
                    (100.0, 100.0),
                ],
                "#1a2748",
            );
        }
        Backdrop::Snow => {
            fill_rect(ctx, k, 0.0, 0.0, 100.0, 100.0, "#d6e8f8");
            poly(
                ctx,
                k,
                &[
                    (0.0, 100.0),
                    (0.0, 70.0),
                    (24.0, 78.0),
                    (46.0, 64.0),
                    (70.0, 80.0),
                    (100.0, 62.0),
                    (100.0, 100.0),
                ],
                "#ffffff",
            );
            pine(ctx, k, 14.0, 36.0);
            for (x, y) in [(78.0, 14.0), (88.0, 24.0), (70.0, 28.0), (92.0, 12.0)] {
                ctx.write_circle(x * k, y * k, 1.3 * k, "#ffffff", 0.95);
            }
        }
    }
}

fn draw_face(ctx: &mut Ctx<'_>, k: f32, face: &Face) {
    ctx.write_circle(
        (face.lx - 6.2) * k,
        (face.ey + 8.0) * k,
        3.7 * k,
        BLUSH,
        0.34,
    );
    ctx.write_circle(
        (face.rx + 6.2) * k,
        (face.ey + 8.0) * k,
        3.7 * k,
        BLUSH,
        0.34,
    );
    solid_eye(ctx, k, face.lx, face);
    solid_eye(ctx, k, face.rx, face);
    smile(ctx, k, face, 6.4, 3.4, INK);
}

fn solid_eye(ctx: &mut Ctx<'_>, k: f32, x: f32, face: &Face) {
    let rx = face.ex;
    let ry = face.ey_r;
    ctx.out().push_str(&format!(
        r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{INK}"/>"#,
        n(x * k),
        n(face.ey * k),
        n(rx * k),
        n(ry * k)
    ));
    ctx.write_circle(
        (x - rx * 0.28) * k,
        (face.ey - ry * 0.32) * k,
        rx * 0.22 * k,
        PAPER,
        0.85,
    );
    if ctx.animated {
        let blink = format_dur(3.4 + f32::from(ctx.hash[27] % 10) * 0.2);
        ctx.out().push_str(&format!(
            r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{tiny}" fill="{FACE}"><animate attributeName="ry" values="{tiny};{tiny};{ry};{tiny};{tiny}" keyTimes="0;0.42;0.48;0.54;1" dur="{blink}s" repeatCount="indefinite"/></ellipse>"#,
            cx = n(x * k),
            cy = n(face.ey * k),
            rx = n(rx * k),
            ry = n(ry * k),
            tiny = n(0.2 * k),
        ));
    }
}

fn smile(ctx: &mut Ctx<'_>, k: f32, face: &Face, w: f32, depth: f32, color: &str) {
    let mut p = Path::new(k);
    p.m(face.mx - w, face.my);
    p.q(face.mx, face.my + depth, face.mx + w, face.my);
    ctx.out().push_str(&format!(
        r#"<path d="{}" fill="none" stroke="{color}" stroke-width="{}" stroke-linecap="round"/>"#,
        p.d,
        n(1.8 * k)
    ));
}

fn draw_glasses(ctx: &mut Ctx<'_>, spec: &Spec, k: f32, face: &Face, cols: &Colors) {
    if spec.glasses == Glasses::None {
        return;
    }
    let frame = cols.fg;
    let sw = n(1.7 * k);
    let y = face.ey;
    match spec.glasses {
        Glasses::None => {}
        Glasses::Round => {
            ctx.out().push_str(&format!(
                r#"<g fill="none" stroke="{frame}" stroke-width="{sw}" stroke-linecap="round">"#
            ));
            ctx.out().push_str(&format!(
                r#"<circle cx="{}" cy="{}" r="{}"/><circle cx="{}" cy="{}" r="{}"/>"#,
                n(face.lx * k),
                n(y * k),
                n(face.lens * k),
                n(face.rx * k),
                n(y * k),
                n(face.lens * k)
            ));
            bridge(ctx, k, face, y);
            temples(ctx, k, face, y);
            ctx.out().push_str("</g>");
        }
        Glasses::Square => {
            ctx.out().push_str(&format!(
                r#"<g fill="none" stroke="{frame}" stroke-width="{sw}" stroke-linejoin="round" stroke-linecap="round">"#
            ));
            lens_rect(ctx, face.lx, y, face.lens * 1.15, face.lens * 0.92, 1.6, "");
            lens_rect(ctx, face.rx, y, face.lens * 1.15, face.lens * 0.92, 1.6, "");
            bridge(ctx, k, face, y);
            temples(ctx, k, face, y);
            ctx.out().push_str("</g>");
        }
        Glasses::Sun => {
            let lens_fill =
                format!(r#"fill="{LENS}" fill-opacity="0.9" stroke="{frame}" stroke-width="{sw}""#);
            lens_rect(
                ctx,
                face.lx,
                y,
                face.lens * 1.25,
                face.lens * 0.95,
                2.2,
                &lens_fill,
            );
            lens_rect(
                ctx,
                face.rx,
                y,
                face.lens * 1.25,
                face.lens * 0.95,
                2.2,
                &lens_fill,
            );
            ctx.out().push_str(&format!(
                r#"<g fill="none" stroke="{frame}" stroke-width="{}" stroke-linecap="round">"#,
                n(2.0 * k)
            ));
            bridge(ctx, k, face, y);
            temples(ctx, k, face, y);
            ctx.out().push_str("</g>");
            // Sheen on each lens.
            for x in [face.lx, face.rx] {
                let mut p = Path::new(k);
                p.m(x - face.lens * 0.7, y - face.lens * 0.15);
                p.l(x - face.lens * 0.15, y - face.lens * 0.55);
                ctx.out().push_str(&format!(
                    r#"<path d="{}" fill="none" stroke="{PAPER}" stroke-width="{}" stroke-linecap="round" stroke-opacity="0.55"/>"#,
                    p.d,
                    n(1.1 * k)
                ));
            }
        }
        Glasses::Cat => {
            ctx.out().push_str(&format!(
                r#"<g fill="none" stroke="{frame}" stroke-width="{sw}" stroke-linecap="round">"#
            ));
            ctx.out().push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}"/>"#,
                n(face.lx * k),
                n(y * k),
                n(face.lens * k),
                n(face.lens * 0.82 * k),
                n(face.rx * k),
                n(y * k),
                n(face.lens * k),
                n(face.lens * 0.82 * k)
            ));
            wing(ctx, k, face.lx, y, face.lens, -1.0);
            wing(ctx, k, face.rx, y, face.lens, 1.0);
            bridge(ctx, k, face, y);
            temples(ctx, k, face, y);
            ctx.out().push_str("</g>");
        }
        Glasses::Monocle => {
            let (x, dir) = if spec.monocle_left() {
                (face.lx, -1.0)
            } else {
                (face.rx, 1.0)
            };
            ctx.out().push_str(&format!(
                r#"<g fill="none" stroke="{frame}" stroke-width="{}" stroke-linecap="round">"#,
                n(1.9 * k)
            ));
            ctx.out().push_str(&format!(
                r#"<circle cx="{}" cy="{}" r="{}"/>"#,
                n(x * k),
                n(y * k),
                n((face.lens + 0.6) * k)
            ));
            let mut chain = Path::new(k);
            chain.m(x + dir * 1.5, y + face.lens + 0.4);
            chain.q(
                x + dir * 4.0,
                y + face.lens + 6.0,
                x + dir * 1.2,
                y + face.lens + 11.0,
            );
            ctx.out().push_str(&format!(
                r#"<path d="{}" stroke-width="{}"/>"#,
                chain.d,
                n(1.1 * k)
            ));
            ctx.write_circle(
                (x + dir * 1.2) * k,
                (y + face.lens + 11.0) * k,
                1.15 * k,
                cols.jewel,
                1.0,
            );
            ctx.out().push_str("</g>");
        }
        Glasses::Aviator => {
            let lens = format!(
                r#"fill="{LENS}" fill-opacity="0.55" stroke="{frame}" stroke-width="{}""#,
                n(1.5 * k)
            );
            let mut left = Path::new(k);
            aviator(&mut left, face.lx, y, -1.0);
            let mut right = Path::new(k);
            aviator(&mut right, face.rx, y, 1.0);
            ctx.write_path(&left.d, &lens);
            ctx.write_path(&right.d, &lens);
            ctx.out().push_str(&format!(
                r#"<g fill="none" stroke="{frame}" stroke-width="{}" stroke-linecap="round">"#,
                n(1.6 * k)
            ));
            ctx.out().push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                n((face.lx + 2.8) * k),
                n((y - 5.6) * k),
                n((face.rx - 2.8) * k),
                n((y - 5.6) * k),
                n((face.lx + 2.2) * k),
                n((y - 3.4) * k),
                n((face.rx - 2.2) * k),
                n((y - 3.4) * k)
            ));
            temples(ctx, k, face, y);
            ctx.out().push_str("</g>");
        }
    }
}

fn lens_rect(ctx: &mut Ctx<'_>, cx: f32, cy: f32, hw: f32, hh: f32, rad: f32, attrs: &str) {
    let k = ctx.size / 100.0;
    ctx.out().push_str(&format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" {attrs}/>"#,
        n((cx - hw) * k),
        n((cy - hh) * k),
        n(hw * 2.0 * k),
        n(hh * 2.0 * k),
        n(rad * k)
    ));
}

fn bridge(ctx: &mut Ctx<'_>, k: f32, face: &Face, y: f32) {
    ctx.out().push_str(&format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
        n((face.lx + face.lens * 0.82) * k),
        n(y * k),
        n((face.rx - face.lens * 0.82) * k),
        n(y * k)
    ));
}

fn temples(ctx: &mut Ctx<'_>, k: f32, face: &Face, y: f32) {
    ctx.out().push_str(&format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
        n((face.lx - face.lens) * k),
        n(y * k),
        n((face.lx - face.lens - 6.5) * k),
        n((y - 2.2) * k),
        n((face.rx + face.lens) * k),
        n(y * k),
        n((face.rx + face.lens + 6.5) * k),
        n((y - 2.2) * k)
    ));
}

fn wing(ctx: &mut Ctx<'_>, k: f32, x: f32, y: f32, lens: f32, dir: f32) {
    let mut p = Path::new(k);
    p.m(x + dir * lens * 0.35, y - lens * 0.62);
    p.q(
        x + dir * lens * 1.25,
        y - lens * 1.15,
        x + dir * lens * 1.55,
        y - lens * 0.15,
    );
    ctx.out().push_str(&format!(
        r#"<path d="{}" stroke-width="{}"/>"#,
        p.d,
        n(1.7 * k)
    ));
}

fn aviator(p: &mut Path, ex: f32, ey: f32, dir: f32) {
    p.m(ex - dir * 3.2, ey - 6.2);
    p.c(
        ex - dir * 8.4,
        ey - 7.4,
        ex - dir * 11.2,
        ey - 1.5,
        ex - dir * 10.2,
        ey + 2.6,
    );
    p.c(
        ex - dir * 9.2,
        ey + 7.2,
        ex - dir * 2.4,
        ey + 8.0,
        ex + dir * 1.4,
        ey + 5.2,
    );
    p.c(
        ex + dir * 3.6,
        ey + 3.2,
        ex + dir * 3.4,
        ey - 1.2,
        ex + dir * 1.2,
        ey - 5.2,
    );
    p.c(
        ex - dir * 0.2,
        ey - 7.4,
        ex - dir * 1.4,
        ey - 6.8,
        ex - dir * 3.2,
        ey - 6.2,
    );
    p.z();
}

fn draw_neck(ctx: &mut Ctx<'_>, spec: &Spec, k: f32, m: &Metrics, cols: &Colors) {
    let y = m.neck;
    let cloth = INK;
    match spec.neck {
        Neck::None => {}
        Neck::Scarf => {
            let dir = if spec.cap_right() { 1.0 } else { -1.0 };
            scarf_tail(ctx, 37.0, y - 1.0, 9.0, 16.0, -16.0 * dir, cols.accent);
            scarf_tail(ctx, 61.0, y + 1.0, 9.0, 22.0, 18.0 * dir, cols.accent);
            ctx.out().push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
                n(33.0 * k),
                n((y - 4.2) * k),
                n(34.0 * k),
                n(9.0 * k),
                n(4.2 * k),
                cols.accent,
                cols.jewel,
                n(1.3 * k)
            ));
        }
        Neck::Bowtie => {
            let mut left = Path::new(k);
            left.m(50.0, y);
            left.l(36.0, y - 6.5);
            left.l(37.5, y + 6.5);
            left.z();
            let mut right = Path::new(k);
            right.m(50.0, y);
            right.l(64.0, y - 6.5);
            right.l(62.5, y + 6.5);
            right.z();
            ctx.write_path(&left.d, &format!(r#"fill="{}""#, cols.accent));
            ctx.write_path(&right.d, &format!(r#"fill="{}""#, cols.accent));
            ctx.write_circle(50.0 * k, y * k, 3.1 * k, cols.fg, 1.0);
        }
        Neck::Necklace => {
            let mut chain = Path::new(k);
            chain.m(34.0, y - 12.0);
            chain.q(50.0, y + 2.0, 66.0, y - 12.0);
            ctx.out().push_str(&format!(
                r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="round"/>"#,
                chain.d,
                cols.fg,
                n(1.5 * k)
            ));
            pendant(ctx, k, 50.0, y + 2.4, spec.pendant(), cols);
        }
        Neck::Collar => {
            let mut p = Path::new(k);
            p.m(34.0, y - 6.0);
            p.l(44.0, y + 8.0);
            p.l(50.0, y + 1.5);
            p.l(56.0, y + 8.0);
            p.l(66.0, y - 6.0);
            p.l(58.0, y - 8.0);
            p.l(50.0, y - 1.0);
            p.l(42.0, y - 8.0);
            p.z();
            ctx.write_path(
                &p.d,
                &format!(
                    r#"fill="{cloth}" stroke="{}" stroke-width="{}" stroke-linejoin="round""#,
                    cols.fg,
                    n(0.8 * k)
                ),
            );
            ctx.write_circle(50.0 * k, (y + 3.2) * k, 1.5 * k, cols.jewel, 1.0);
        }
        Neck::Tie => {
            let mut knot = Path::new(k);
            knot.m(45.4, y - 5.0);
            knot.l(54.6, y - 5.0);
            knot.l(52.8, y + 2.2);
            knot.l(47.2, y + 2.2);
            knot.z();
            let mut blade = Path::new(k);
            blade.m(46.6, y + 1.4);
            blade.l(53.4, y + 1.4);
            blade.l(56.2, y + 16.0);
            blade.l(50.0, y + 22.0);
            blade.l(43.8, y + 16.0);
            blade.z();
            ctx.write_path(&blade.d, &format!(r#"fill="{}""#, cols.accent));
            let mut stripe = Path::new(k);
            stripe.m(48.2, y + 5.5);
            stripe.l(52.2, y + 8.0);
            stripe.l(50.6, y + 17.5);
            stripe.l(46.8, y + 14.8);
            stripe.z();
            ctx.write_path(
                &stripe.d,
                &format!(r#"fill="{}" fill-opacity="0.85""#, cols.jewel),
            );
            ctx.write_path(&knot.d, &format!(r#"fill="{}""#, cols.accent));
            ctx.out().push_str(&format!(
                r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linejoin="round"/>"#,
                knot.d,
                cols.fg,
                n(0.7 * k)
            ));
        }
        Neck::Bandana => {
            let mut left = Path::new(k);
            left.m(48.0, y - 1.0);
            left.l(33.0, y + 5.0);
            left.l(39.0, y + 16.0);
            left.z();
            let mut right = Path::new(k);
            right.m(52.0, y - 1.0);
            right.l(67.0, y + 7.0);
            right.l(58.0, y + 18.0);
            right.z();
            let fill = format!(
                r#"fill="{}" stroke="{}" stroke-width="{}" stroke-linejoin="round""#,
                cols.accent,
                cols.fg,
                n(0.7 * k)
            );
            ctx.write_path(&left.d, &fill);
            ctx.write_path(&right.d, &fill);
            ctx.out().push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}"/>"#,
                n(40.0 * k),
                n((y - 5.5) * k),
                n(20.0 * k),
                n(7.0 * k),
                n(3.2 * k),
                cols.accent
            ));
            ctx.write_circle(50.0 * k, (y - 1.6) * k, 3.4 * k, cols.jewel, 1.0);
        }
        Neck::Choker => {
            let mut p = Path::new(k);
            p.m(34.0, y - 1.0);
            p.q(50.0, y + 5.5, 66.0, y - 1.0);
            ctx.out().push_str(&format!(
                r#"<path d="{}" fill="none" stroke="{cloth}" stroke-width="{}" stroke-linecap="round"/>"#,
                p.d,
                n(3.3 * k)
            ));
            let mut gem = Path::new(k);
            gem.m(50.0, y + 0.2);
            gem.l(53.0, y + 3.4);
            gem.l(50.0, y + 6.8);
            gem.l(47.0, y + 3.4);
            gem.z();
            ctx.write_path(
                &gem.d,
                &format!(
                    r#"fill="{}" stroke="{}" stroke-width="{}""#,
                    cols.jewel,
                    cols.fg,
                    n(0.6 * k)
                ),
            );
        }
    }
}

fn scarf_tail(ctx: &mut Ctx<'_>, x: f32, y: f32, w: f32, h: f32, angle: f32, color: &str) {
    let k = ctx.size / 100.0;
    ctx.out().push_str(&format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{color}" transform="rotate({} {} {})"/>"#,
        n((x - w * 0.5) * k),
        n(y * k),
        n(w * k),
        n(h * k),
        n(2.4 * k),
        n(angle),
        n(x * k),
        n(y * k)
    ));
    // Fringe.
    for i in 0..3 {
        let fx = x - w * 0.28 + i as f32 * w * 0.28;
        ctx.out().push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{color}" stroke-width="{}" stroke-linecap="round" transform="rotate({} {} {})"/>"#,
            n(fx * k),
            n((y + h - 0.4) * k),
            n(fx * k),
            n((y + h + 2.4) * k),
            n(0.9 * k),
            n(angle),
            n(x * k),
            n(y * k)
        ));
    }
}

fn pendant(ctx: &mut Ctx<'_>, k: f32, x: f32, y: f32, kind: u8, cols: &Colors) {
    match kind {
        0 => {
            ctx.write_circle(x * k, (y + 5.6) * k, 4.4 * k, cols.jewel, 1.0);
            ctx.write_ring(x * k, (y + 5.6) * k, 4.4 * k, cols.fg, 0.9 * k, 1.0);
        }
        1 => {
            let mut p = Path::new(k);
            p.m(x, y + 1.2);
            p.l(x + 4.6, y + 6.4);
            p.l(x, y + 11.6);
            p.l(x - 4.6, y + 6.4);
            p.z();
            ctx.write_path(
                &p.d,
                &format!(
                    r#"fill="{}" stroke="{}" stroke-width="{}""#,
                    cols.jewel,
                    cols.fg,
                    n(0.8 * k)
                ),
            );
        }
        _ => {
            let pts = star_polygon(x * k, (y + 6.0) * k, 5.6 * k, 2.4 * k, 5, -FRAC_PI_2);
            let mut d = String::with_capacity(80);
            polygon_d(&mut d, &pts);
            ctx.write_path(
                &d,
                &format!(
                    r#"fill="{}" stroke="{}" stroke-width="{}""#,
                    cols.jewel,
                    cols.fg,
                    n(0.7 * k)
                ),
            );
        }
    }
}

fn draw_hat(ctx: &mut Ctx<'_>, spec: &Spec, k: f32, m: &Metrics, cols: &Colors) {
    if spec.hat == Hat::None {
        return;
    }
    let y = m.crown;
    let hw = m.half;
    match spec.hat {
        Hat::None => {}
        Hat::Beanie => {
            let dome = rise(m, 0.95);
            let pom_r = (hw * 0.26).clamp(3.3, 5.2);
            ctx.write_circle(
                50.0 * k,
                (y - dome - pom_r * 0.15) * k,
                pom_r * k,
                cols.jewel,
                1.0,
            );
            ctx.write_circle(
                (50.0 - pom_r * 0.3) * k,
                (y - dome - pom_r * 0.45) * k,
                pom_r * 0.28 * k,
                PAPER,
                0.55,
            );
            let mut dome_p = Path::new(k);
            dome_p.m(50.0 - hw, y + 2.0);
            dome_p.c(
                50.0 - hw,
                y - dome * 0.15,
                50.0 - hw * 0.72,
                y - dome,
                50.0,
                y - dome,
            );
            dome_p.c(
                50.0 + hw * 0.72,
                y - dome,
                50.0 + hw,
                y - dome * 0.15,
                50.0 + hw,
                y + 2.0,
            );
            dome_p.z();
            ctx.write_path(&dome_p.d, &format!(r#"fill="{}""#, cols.accent));
            ctx.out().push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}"/>"#,
                n((50.0 - hw - 1.0) * k),
                n((y - 0.5) * k),
                n((hw * 2.0 + 2.0) * k),
                n(6.2 * k),
                n(2.0 * k),
                cols.accent
            ));
            for i in 0..4 {
                let x = 50.0 - hw + 3.0 + i as f32 * (hw * 2.0 - 4.0) / 3.0;
                ctx.out().push_str(&format!(
                    r#"<line x1="{x}" y1="{y0}" x2="{x}" y2="{y1}" stroke="{}" stroke-width="{}" stroke-linecap="round" stroke-opacity="0.45"/>"#,
                    cols.jewel,
                    n(1.0 * k),
                    x = n(x * k),
                    y0 = n(y * k),
                    y1 = n((y + 5.2) * k),
                ));
            }
        }
        Hat::Top => {
            let h = rise(m, 1.15);
            let cw = (hw * 0.72).min(14.0);
            ctx.out().push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/>"#,
                n(50.0 * k),
                n((y + 3.0) * k),
                n((hw + 3.0) * k),
                n(4.2 * k),
                cols.accent
            ));
            let mut crown = Path::new(k);
            crown.m(50.0 - cw, y + 1.0);
            crown.l(50.0 - cw + 1.4, y - h);
            crown.l(50.0 + cw - 1.4, y - h);
            crown.l(50.0 + cw, y + 1.0);
            crown.z();
            ctx.write_path(&crown.d, &format!(r#"fill="{}""#, cols.accent));
            ctx.out().push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/>"#,
                n(50.0 * k),
                n((y - h) * k),
                n((cw - 1.2) * k),
                n(2.6 * k),
                cols.accent
            ));
            ctx.out().push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
                n((50.0 - cw + 0.4) * k),
                n((y - h * 0.38) * k),
                n((cw * 2.0 - 0.8) * k),
                n(3.6 * k),
                cols.jewel
            ));
        }
        Hat::Cap => {
            let dir = if spec.cap_right() { 1.0 } else { -1.0 };
            let dome = rise(m, 0.72);
            let mut dome_p = Path::new(k);
            dome_p.m(50.0 - hw * 0.85, y + 3.0);
            dome_p.q(50.0 - hw * 0.9, y - dome, 50.0, y - dome);
            dome_p.q(50.0 + hw * 0.9, y - dome, 50.0 + hw * 0.85, y + 3.0);
            dome_p.z();
            ctx.write_path(&dome_p.d, &format!(r#"fill="{}""#, cols.accent));
            ctx.out().push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" transform="rotate({} {} {})"/>"#,
                n((50.0 + dir * hw * 0.85) * k),
                n((y + 4.2) * k),
                n((hw * 0.95) * k),
                n(4.4 * k),
                cols.jewel,
                n(dir * -8.0),
                n((50.0 + dir * hw * 0.55) * k),
                n((y + 3.0) * k)
            ));
            ctx.write_circle(50.0 * k, (y - dome) * k, 1.7 * k, cols.jewel, 1.0);
        }
        Hat::Beret => {
            let dir = if spec.cap_right() { -1.0 } else { 1.0 };
            let cy = y - 1.0;
            ctx.out().push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" transform="rotate({} {} {})"/>"#,
                n((50.0 + dir * 3.0) * k),
                n(cy * k),
                n((hw + 2.0) * k),
                n(7.2 * k),
                cols.accent,
                n(dir * 16.0),
                n(50.0 * k),
                n(cy * k)
            ));
            ctx.write_circle(
                (50.0 + dir * 2.0) * k,
                (cy - 5.5) * k,
                1.8 * k,
                cols.jewel,
                1.0,
            );
        }
        Hat::Crown => {
            let h = rise(m, 0.85);
            let left = 50.0 - hw;
            let span = hw * 2.0;
            let teeth = 5;
            for i in 0..teeth {
                let x0 = left + span * (i as f32) / teeth as f32;
                let x1 = left + span * (i as f32 + 1.0) / teeth as f32;
                let mid = (x0 + x1) * 0.5;
                let mut p = Path::new(k);
                p.m(x0 + 0.4, y + 1.0);
                p.l(mid, y - h);
                p.l(x1 - 0.4, y + 1.0);
                p.z();
                ctx.write_path(&p.d, &format!(r#"fill="{}""#, cols.accent));
                ctx.write_circle(mid * k, (y - h + 2.4) * k, 1.35 * k, cols.jewel, 1.0);
            }
            ctx.out().push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}"/>"#,
                n((left - 0.5) * k),
                n((y - 1.0) * k),
                n((span + 1.0) * k),
                n(6.4 * k),
                n(1.4 * k),
                cols.accent
            ));
            for i in 0..3 {
                let x = 50.0 + (i as f32 - 1.0) * hw * 0.55;
                ctx.write_circle(x * k, (y + 2.2) * k, 1.45 * k, cols.jewel, 1.0);
            }
        }
        Hat::Party => {
            let h = rise(m, 1.2);
            let base = hw * 0.72;
            let mut cone = Path::new(k);
            cone.m(50.0 - base, y + 2.0);
            cone.l(50.0, y - h);
            cone.l(50.0 + base, y + 2.0);
            cone.z();
            ctx.write_path(&cone.d, &format!(r#"fill="{}""#, cols.accent));
            let mut stripe = Path::new(k);
            let sy = y - h * 0.42;
            stripe.m(50.0 - base * 0.5, sy);
            stripe.l(50.0 + base * 0.5, sy - 1.5);
            stripe.l(50.0 + base * 0.42, sy + 2.4);
            stripe.l(50.0 - base * 0.42, sy + 3.6);
            stripe.z();
            ctx.write_path(&stripe.d, &format!(r#"fill="{}""#, cols.jewel));
            ctx.write_circle(50.0 * k, (y - h) * k, 3.3 * k, cols.jewel, 1.0);
        }
        Hat::Bow => {
            let cy = y - 2.0;
            ctx.out().push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" transform="rotate(-24 {} {})"/>"#,
                n((50.0 - 7.2) * k),
                n(cy * k),
                n(8.2 * k),
                n(5.4 * k),
                cols.accent,
                n((50.0 - 7.2) * k),
                n(cy * k)
            ));
            ctx.out().push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" transform="rotate(24 {} {})"/>"#,
                n((50.0 + 7.2) * k),
                n(cy * k),
                n(8.2 * k),
                n(5.4 * k),
                cols.accent,
                n((50.0 + 7.2) * k),
                n(cy * k)
            ));
            for dir in [-1.0_f32, 1.0] {
                let mut tail = Path::new(k);
                tail.m(50.0 + dir * 1.4, cy + 1.5);
                tail.l(50.0 + dir * 7.5, cy + 12.0);
                tail.l(50.0 + dir * 2.2, cy + 10.5);
                tail.z();
                ctx.write_path(&tail.d, &format!(r#"fill="{}""#, cols.accent));
            }
            ctx.write_circle(50.0 * k, cy * k, 3.2 * k, cols.jewel, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{Theme, palette_by_theme};
    use crate::{Engine, RenderOptions, render_identicon_inner};
    use std::collections::BTreeSet;

    fn attr(svg: &str, key: &str) -> String {
        let pat = format!("{key}=\"");
        let start = svg.find(&pat).unwrap_or_else(|| panic!("missing {key}")) + pat.len();
        let end = start + svg[start..].find('"').unwrap();
        svg[start..end].to_string()
    }

    fn hash_of(seed: u8) -> [u8; 32] {
        let mut hash = [40u8; 32];
        hash[30] = seed;
        hash
    }

    fn paint_svg(spec: &Spec, seed: u8) -> String {
        let hash = hash_of(seed);
        let palette = palette_by_theme(Theme::Nord);
        let mut ctx = Ctx::new(220, palette, &hash, false, spec.bg.is_some());
        paint(&mut ctx, spec);
        ctx.finish();
        ctx.into_string()
    }

    fn baseline() -> Spec {
        Spec {
            hat: Hat::None,
            glasses: Glasses::None,
            neck: Neck::None,
            bg: Some(Backdrop::Sky),
            flip: 0,
        }
    }

    fn ink_eyes(svg: &str) -> String {
        let mut eyes = String::new();
        let mut rest = svg;
        while let Some(i) = rest.find("<ellipse") {
            let end = i + rest[i..].find("/>").unwrap() + 2;
            let el = &rest[i..end];
            if el.contains(r##"fill="#1c1917""##) {
                eyes.push_str(el);
            }
            rest = &rest[end..];
        }
        eyes
    }

    #[test]
    fn each_variant_changes_the_picture() {
        let bare = paint_svg(&baseline(), 1);
        assert!(bare.contains(r#"data-shape="round""#));
        assert!(bare.contains(r#"data-mood="smile""#));
        for hat in Hat::ALL.into_iter().filter(|h| *h != Hat::None) {
            let svg = paint_svg(&Spec { hat, ..baseline() }, 3);
            assert_ne!(svg, bare, "{}", hat.name());
        }
        for glasses in Glasses::ALL.into_iter().filter(|g| *g != Glasses::None) {
            let svg = paint_svg(
                &Spec {
                    glasses,
                    ..baseline()
                },
                4,
            );
            assert_ne!(svg, bare, "{}", glasses.name());
        }
        for neck in Neck::ALL.into_iter().filter(|n| *n != Neck::None) {
            let svg = paint_svg(&Spec { neck, ..baseline() }, 5);
            assert_ne!(svg, bare, "{}", neck.name());
        }
        for bg in Backdrop::ALL.into_iter().filter(|b| *b != Backdrop::Sky) {
            let svg = paint_svg(
                &Spec {
                    bg: Some(bg),
                    ..baseline()
                },
                6,
            );
            assert_ne!(svg, bare, "{}", bg.name());
            assert!(svg.contains(&format!(r#"data-bg="{}""#, bg.name())));
        }
    }

    #[test]
    fn transparent_background_omits_the_badge() {
        let opts = RenderOptions {
            engine: Engine::Character,
            background: false,
            ..RenderOptions::default()
        };
        let svg = render_identicon_inner("alice", &opts).unwrap();
        assert!(svg.contains(r#"data-bg="none""#));
        assert!(!svg.contains("#7ec8f8"));
        assert!(!svg.contains("#1277b8"));
    }

    #[test]
    fn names_cover_every_slot() {
        let mut shapes = BTreeSet::new();
        let mut hats = BTreeSet::new();
        let mut glasses = BTreeSet::new();
        let mut necks = BTreeSet::new();
        let mut backs = BTreeSet::new();
        let mut moods = BTreeSet::new();
        let opts = RenderOptions {
            engine: Engine::Character,
            ..RenderOptions::default()
        };
        let eyes = ink_eyes(&render_identicon_inner("alice", &opts).unwrap());
        assert!(eyes.contains("<ellipse"));
        for i in 0..240 {
            let name = format!("person-{i}");
            let svg = render_identicon_inner(&name, &opts).unwrap();
            shapes.insert(attr(&svg, "data-shape"));
            hats.insert(attr(&svg, "data-hat"));
            glasses.insert(attr(&svg, "data-glasses"));
            necks.insert(attr(&svg, "data-neck"));
            backs.insert(attr(&svg, "data-bg"));
            moods.insert(attr(&svg, "data-mood"));
            assert_eq!(ink_eyes(&svg), eyes, "{name}");
            assert!(svg.contains(r##"fill="#8fd0ef""##), "{name}");
            assert!(svg.len() < 12 * 1024, "{}", svg.len());
        }
        assert_eq!(shapes.into_iter().collect::<Vec<_>>(), vec!["round"]);
        assert_eq!(hats.len(), Hat::ALL.len());
        assert_eq!(glasses.len(), Glasses::ALL.len());
        assert_eq!(necks.len(), Neck::ALL.len());
        assert_eq!(backs.len(), Backdrop::ALL.len());
        assert_eq!(moods.into_iter().collect::<Vec<_>>(), vec!["smile"]);
    }
}
