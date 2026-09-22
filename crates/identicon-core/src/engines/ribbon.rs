//! Interlaced ribbons following Lissajous curves. The frequency ratio and the
//! ribbon count define the figure; phases, widths and colours add detail.
//! Over/under weaving is obtained by drawing curve segments in depth order,
//! each with a background-coloured underlay.

use super::common::Ctx;
use crate::bezier::write_f32;
use crate::prng::Prng;
use std::f32::consts::{PI, TAU};

const SAMPLES: usize = 64;
const SEGMENTS: usize = 8;

struct Segment {
    ribbon: usize,
    depth: f32,
    d: String,
}

pub fn render(ctx: &mut Ctx, prng: &mut Prng) {
    let s = ctx.size;
    let c = ctx.center();

    let ribbon_count = 1 + ctx.trait_u8(0, 3) as usize;
    let (fa, fb) = [(1u32, 2u32), (2, 3), (3, 4), (1, 3), (3, 5)][ctx.trait_u8(1, 5) as usize];
    let [ca, cb, cc] = Ctx::distinct3(u32::from(ctx.hash[2]) | (u32::from(ctx.hash[3]) << 8));
    let width = s * (0.06 + 0.02 * (3 - ribbon_count) as f32);
    let colors = [(ca, cb), (cb, cc), (cc, ca)];

    // Lissajous curves fold onto themselves (an open arc traced twice) for
    // phases at `degenerate0 + k*PI/fb`; we stay half a period away from them.
    let degenerate0 = match (fa, fb) {
        (1, 2) => PI / 4.0,
        (2, 3) => PI / 6.0,
        (3, 4) => PI / 8.0,
        _ => 0.0,
    };
    let period = PI / fb as f32;

    let mut segments: Vec<Segment> = Vec::with_capacity(ribbon_count * SEGMENTS);
    for r in 0..ribbon_count {
        let k = prng.range_u32(0, fb * 2 - 1) as f32;
        let phase = degenerate0 + period * (k + 0.5) + prng.range_f32(-0.1, 0.1) * period;
        let depth_phase = prng.range_f32(0.0, TAU);
        let amp = s * 0.36 * (1.0 - 0.1 * r as f32);
        let tilt = prng.range_f32(-0.25, 0.25);
        let pts: Vec<(f32, f32)> = (0..SAMPLES)
            .map(|i| {
                let t = TAU * i as f32 / SAMPLES as f32;
                let x = amp * (fa as f32 * t + phase).sin();
                let y = amp * 0.92 * (fb as f32 * t).sin();
                (
                    c + x * tilt.cos() - y * tilt.sin(),
                    c + x * tilt.sin() + y * tilt.cos(),
                )
            })
            .collect();

        let per = SAMPLES / SEGMENTS;
        for k in 0..SEGMENTS {
            let idx: Vec<usize> = (0..=per).map(|j| (k * per + j) % SAMPLES).collect();
            let mid_t = TAU * (k as f32 + 0.5) / SEGMENTS as f32;
            let depth = (2.0 * mid_t + depth_phase).sin() + 0.01 * r as f32;
            let mut d = String::with_capacity(idx.len() * 40);
            catmull_rom_open(&mut d, &pts, &idx);
            segments.push(Segment {
                ribbon: r,
                depth,
                d,
            });
        }
    }
    segments.sort_by(|a, b| {
        a.depth
            .partial_cmp(&b.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ctx.begin();
    for (r, (c0, c1)) in colors.iter().take(ribbon_count).enumerate() {
        let id = ctx.id(&format!("rg{r}"));
        let color0 = ctx.color(*c0);
        let color1 = ctx.color(*c1);
        let (x1, x2) = if r % 2 == 0 {
            ("0".to_string(), fmt(s))
        } else {
            (fmt(s), "0".to_string())
        };
        let animated = ctx.animated;
        let out = ctx.out();
        out.push_str(&format!(
            r#"<linearGradient id="{id}" gradientUnits="userSpaceOnUse" x1="{x1}" y1="0" x2="{x2}" y2="{}"><stop offset="0" stop-color="{color0}"/><stop offset="1" stop-color="{color1}"/>"#,
            fmt(s)
        ));
        if animated {
            out.push_str(&format!(
                r#"<animate attributeName="y1" values="0;{};0" dur="8s" repeatCount="indefinite"/>"#,
                fmt(s * 0.6)
            ));
        }
        out.push_str("</linearGradient>");
    }
    ctx.close_defs();

    let bg = ctx.palette.bg_dark;
    let under_w = fmt(width + s * 0.022);
    let main_w = fmt(width);

    // Underlays use butt caps so a later segment never bites into the previous
    // one at their shared joint; coloured strokes use round caps to close the
    // tiny gap that butt caps would leave.
    ctx.out()
        .push_str(r#"<g fill="none" stroke-linejoin="round">"#);
    for seg in &segments {
        let id = ctx.id(&format!("rg{}", seg.ribbon));
        let out = ctx.out();
        out.push_str(&format!(
            r#"<path d="{}" stroke="{bg}" stroke-width="{under_w}" stroke-linecap="butt"/>"#,
            seg.d
        ));
        out.push_str(&format!(
            r#"<path d="{}" stroke="url(#{id})" stroke-width="{main_w}" stroke-linecap="round"/>"#,
            seg.d
        ));
    }
    ctx.out().push_str("</g>");
}

/// Open Catmull-Rom spline through `pts[idx]`, written as cubic Béziers.
fn catmull_rom_open(out: &mut String, pts: &[(f32, f32)], idx: &[usize]) {
    let n = pts.len();
    let at = |k: isize| -> (f32, f32) {
        let i = idx[0] as isize + k;
        pts[i.rem_euclid(n as isize) as usize]
    };
    let first = at(0);
    out.push('M');
    write_f32(out, first.0);
    out.push(',');
    write_f32(out, first.1);
    for k in 0..(idx.len() as isize - 1) {
        let p0 = at(k - 1);
        let p1 = at(k);
        let p2 = at(k + 1);
        let p3 = at(k + 2);
        let cp1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
        let cp2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
        out.push('C');
        for p in [cp1, cp2, p2] {
            write_f32(out, p.0);
            out.push(',');
            write_f32(out, p.1);
            out.push(' ');
        }
        out.pop();
    }
}

fn fmt(v: f32) -> String {
    let mut s = String::new();
    write_f32(&mut s, v);
    s
}
