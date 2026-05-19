use crate::bezier::write_f32;
use crate::palette::Palette;
use crate::prng::Prng;

const TAU: f32 = std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundStyle {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CenterGlyph {
    Ring,
    Cross,
    Diamond,
    Triad,
    Seal,
    Core,
}

pub struct SigilParams {
    pub fold: u32,
    pub arc_layers: u32,
    pub orbit_rings: u32,
    pub nodes_per_ring: u32,
    pub palette_index: usize,
    pub background_style: BackgroundStyle,
    pub center_glyph: CenterGlyph,
    pub rotation_offset: f32,
    pub stroke_width: f32,
}

impl SigilParams {
    pub fn from_hash_and_prng(hash: &[u8; 32], prng: &mut Prng) -> Self {
        let folds = [4u32, 6, 8, 8, 12, 12];
        let fold = folds[(hash[2] as usize) % folds.len()];

        Self {
            fold,
            arc_layers: prng.range_u32(2, 5),
            orbit_rings: prng.range_u32(1, 3),
            nodes_per_ring: prng.range_u32(3, fold.max(6)),
            palette_index: hash[3] as usize,
            background_style: if hash[5] & 1 == 0 {
                BackgroundStyle::Light
            } else {
                BackgroundStyle::Dark
            },
            center_glyph: match hash[4] % 6 {
                0 => CenterGlyph::Ring,
                1 => CenterGlyph::Cross,
                2 => CenterGlyph::Diamond,
                3 => CenterGlyph::Triad,
                4 => CenterGlyph::Seal,
                _ => CenterGlyph::Core,
            },
            rotation_offset: (hash[6] as f32 / 255.0) * TAU,
            stroke_width: prng.range_f32(1.5, 3.5),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArcLayer {
    pub radius: f32,
    pub start: f32,
    pub sweep: f32,
    pub color_index: usize,
    pub opacity: f32,
    pub stroke_width: f32,
}

#[derive(Clone, Debug)]
pub struct OrbitNode {
    pub radius: f32,
    pub angle: f32,
    pub radius_px: f32,
    pub color_index: usize,
    pub opacity: f32,
}

pub fn generate_arc_layers(prng: &mut Prng, params: &SigilParams, size: f32) -> Vec<ArcLayer> {
    let cx = size * 0.5;
    let max_r = cx * 0.88;
    let min_r = cx * 0.22;
    let sector = TAU / params.fold as f32;
    let mut layers = Vec::new();

    for i in 0..params.arc_layers {
        let t = (i as f32 + 0.35) / (params.arc_layers as f32 + 0.5);
        let radius = min_r + (max_r - min_r) * t;
        let sweep = prng.range_f32(0.25, 0.85) * sector;
        let start = params.rotation_offset + prng.range_f32(0.0, sector - sweep);

        for s in 0..params.fold {
            layers.push(ArcLayer {
                radius,
                start: start + s as f32 * sector,
                sweep,
                color_index: (prng.next_u32() as usize) % 5,
                opacity: prng.range_f32(0.45, 0.95),
                stroke_width: params.stroke_width * prng.range_f32(0.85, 1.15),
            });
        }
    }

    layers
}

pub fn generate_orbit_nodes(prng: &mut Prng, params: &SigilParams, size: f32) -> Vec<OrbitNode> {
    let cx = size * 0.5;
    let mut nodes = Vec::new();

    for ring in 0..params.orbit_rings {
        let t = (ring as f32 + 1.0) / (params.orbit_rings as f32 + 1.2);
        let radius = cx * (0.28 + t * 0.52);
        let count = params.nodes_per_ring + ring;
        let phase = params.rotation_offset + prng.range_f32(0.0, TAU);

        for n in 0..count {
            let angle = phase + (n as f32 / count as f32) * TAU;
            nodes.push(OrbitNode {
                radius,
                angle,
                radius_px: prng.range_f32(2.5, 5.5),
                color_index: (prng.next_u32() as usize) % 5,
                opacity: prng.range_f32(0.5, 0.95),
            });
        }
    }

    nodes
}

pub fn write_arc_path(out: &mut String, cx: f32, cy: f32, layer: &ArcLayer) {
    let a0 = layer.start;
    let a1 = layer.start + layer.sweep;
    let r = layer.radius;

    let (x0, y0) = polar(cx, cy, r, a0);
    let (x1, y1) = polar(cx, cy, r, a1);

    let large = if layer.sweep > std::f32::consts::PI {
        1
    } else {
        0
    };

    out.push_str("M ");
    write_f32(out, x0);
    out.push(',');
    write_f32(out, y0);
    out.push_str(" A ");
    write_f32(out, r);
    out.push(',');
    write_f32(out, r);
    out.push_str(" 0 ");
    out.push(if large == 1 { '1' } else { '0' });
    out.push_str(" 1 ");
    write_f32(out, x1);
    out.push(',');
    write_f32(out, y1);
}

pub fn write_center_glyph(
    out: &mut String,
    glyph: CenterGlyph,
    cx: f32,
    cy: f32,
    size: f32,
    color: &str,
    opacity: f32,
) {
    let u = size * 0.11;
    match glyph {
        CenterGlyph::Ring => {
            write_circle(out, cx, cy, u * 1.1, color, opacity, false);
            write_circle(out, cx, cy, u * 0.45, color, opacity * 0.9, true);
        }
        CenterGlyph::Cross => {
            let arm = u * 1.2;
            write_line(out, cx - arm, cy, cx + arm, cy, color, opacity, u * 0.22);
            write_line(out, cx, cy - arm, cx, cy + arm, color, opacity, u * 0.22);
            write_circle(out, cx, cy, u * 0.35, color, opacity, true);
        }
        CenterGlyph::Diamond => {
            write_polygon(
                out,
                &[
                    (cx, cy - u * 1.3),
                    (cx + u, cy),
                    (cx, cy + u * 1.3),
                    (cx - u, cy),
                ],
                color,
                opacity,
                false,
            );
        }
        CenterGlyph::Triad => {
            for i in 0..3 {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 * TAU / 3.0;
                let (px, py) = polar(cx, cy, u * 0.95, a);
                write_circle(out, px, py, u * 0.38, color, opacity, true);
            }
            write_circle(out, cx, cy, u * 0.28, color, opacity * 0.85, true);
        }
        CenterGlyph::Seal => {
            write_circle(out, cx, cy, u * 1.15, color, opacity * 0.35, false);
            write_polygon(
                out,
                &[
                    (cx, cy - u),
                    (cx + u * 0.87, cy + u * 0.5),
                    (cx - u * 0.87, cy + u * 0.5),
                ],
                color,
                opacity * 0.75,
                false,
            );
            write_circle(out, cx, cy, u * 0.25, color, 1.0, true);
        }
        CenterGlyph::Core => {
            write_circle(out, cx, cy, u * 0.55, color, opacity, true);
            for i in 0..4 {
                let a = i as f32 * std::f32::consts::FRAC_PI_2;
                let (x0, y0) = polar(cx, cy, u * 0.55, a);
                let (x1, y1) = polar(cx, cy, u * 1.05, a);
                write_line(out, x0, y0, x1, y1, color, opacity * 0.7, u * 0.12);
            }
        }
    }
}

fn polar(cx: f32, cy: f32, r: f32, a: f32) -> (f32, f32) {
    (cx + r * a.cos(), cy + r * a.sin())
}

fn write_circle(out: &mut String, cx: f32, cy: f32, r: f32, color: &str, opacity: f32, filled: bool) {
    out.push_str("<circle cx=\"");
    write_f32(out, cx);
    out.push_str("\" cy=\"");
    write_f32(out, cy);
    out.push_str("\" r=\"");
    write_f32(out, r);
    out.push('"');
    if filled {
        out.push_str(" fill=\"");
        out.push_str(color);
        out.push_str("\" fill-opacity=\"");
        write_f32(out, opacity);
        out.push('"');
    } else {
        out.push_str(" fill=\"none\" stroke=\"");
        out.push_str(color);
        out.push_str("\" stroke-width=\"");
        write_f32(out, (r * 0.35).max(1.0));
        out.push_str("\" stroke-opacity=\"");
        write_f32(out, opacity);
        out.push('"');
    }
    out.push_str("/>");
}

#[allow(clippy::too_many_arguments)]
fn write_line(
    out: &mut String,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: &str,
    opacity: f32,
    width: f32,
) {
    out.push_str("<line x1=\"");
    write_f32(out, x0);
    out.push_str("\" y1=\"");
    write_f32(out, y0);
    out.push_str("\" x2=\"");
    write_f32(out, x1);
    out.push_str("\" y2=\"");
    write_f32(out, y1);
    out.push_str("\" stroke=\"");
    out.push_str(color);
    out.push_str("\" stroke-width=\"");
    write_f32(out, width);
    out.push_str("\" stroke-opacity=\"");
    write_f32(out, opacity);
    out.push_str("\" stroke-linecap=\"round\"/>");
}

fn write_polygon(out: &mut String, pts: &[(f32, f32)], color: &str, opacity: f32, stroke: bool) {
    out.push_str("<polygon points=\"");
    for (i, (x, y)) in pts.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        write_f32(out, *x);
        out.push(',');
        write_f32(out, *y);
    }
    out.push_str("\" fill=\"");
    out.push_str(color);
    out.push_str("\" fill-opacity=\"");
    write_f32(out, opacity);
    if stroke {
        out.push_str("\" stroke=\"");
        out.push_str(color);
        out.push_str("\" stroke-width=\"1\"");
    } else {
        out.push('"');
    }
    out.push_str("/>");
}

pub fn palette_color(palette: &Palette, index: usize) -> &'static str {
    palette.colors[index % palette.colors.len()]
}
