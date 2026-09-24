//! Kaleidoscopic mandala: one asymmetric petal repeated N times, optionally
//! mirrored. N and the mirror flag define the silhouette (cyclic pinwheel vs
//! dihedral flower); the petal curve carries the detail.

use super::common::{Ctx, smooth_closed_d};
use crate::bezier::write_f32;
use crate::prng::Prng;
use std::f32::consts::TAU;

pub fn render(ctx: &mut Ctx, prng: &mut Prng) {
    let s = ctx.size;
    let c = ctx.center();

    let folds = [3u32, 4, 5, 6, 8][ctx.trait_u8(0, 5) as usize];
    let mirror = ctx.trait_u8(1, 2) == 0;
    let [ca, cb, cc] = Ctx::distinct3(u32::from(ctx.hash[2]) | (u32::from(ctx.hash[3]) << 8));
    let two_layers = ctx.trait_u8(4, 4) != 0;

    let outer = petal(prng, s * 0.44, s * 0.11, 0.3);
    let inner = petal(prng, s * 0.27, s * 0.06, 0.45);

    let outer_id = ctx.id("po");
    let inner_id = ctx.id("pi");

    ctx.begin();
    {
        let color_a = ctx.color(ca);
        let color_b = ctx.color(cb);
        let color_c = ctx.color(cc);
        let stroke_w = fmt(s * 0.005);
        let out = ctx.out();
        out.push_str(&format!(
            r#"<path id="{outer_id}" d="{outer}" fill="{color_a}" fill-opacity="0.62" stroke="{color_c}" stroke-opacity="0.9" stroke-width="{stroke_w}"/>"#
        ));
        out.push_str(&format!(
            r#"<path id="{inner_id}" d="{inner}" fill="{color_b}" fill-opacity="0.78" stroke="{color_c}" stroke-opacity="0.6" stroke-width="{stroke_w}"/>"#
        ));
    }
    ctx.close_defs();

    let color_c = ctx.color(cc);
    let color_a = ctx.color(ca);

    ctx.write_ring(c, c, s * 0.46, color_c, s * 0.004, 0.35);

    ctx.out().push_str(&format!(
        r#"<g transform="translate({} {})">"#,
        fmt(c),
        fmt(c)
    ));
    if ctx.animated {
        ctx.out().push_str(
            r#"<g><animateTransform attributeName="transform" type="rotate" from="0" to="360" dur="90s" repeatCount="indefinite"/>"#,
        );
    }

    let step = 360.0 / folds as f32;
    write_layer(ctx, &outer_id, folds, step, 0.0, mirror);
    if two_layers {
        write_layer(ctx, &inner_id, folds, step, step * 0.5, mirror);
    }

    if ctx.animated {
        ctx.out().push_str("</g>");
    }
    ctx.out().push_str("</g>");

    ctx.write_circle(c, c, s * 0.055, ctx.palette.bg_dark, 0.9);
    ctx.write_circle(c, c, s * 0.035, color_a, 1.0);
    ctx.write_ring(c, c, s * 0.055, color_c, s * 0.006, 0.9);
}

fn write_layer(ctx: &mut Ctx, id: &str, folds: u32, step: f32, offset: f32, mirror: bool) {
    for k in 0..folds {
        let angle = fmt(offset + step * k as f32);
        let out = ctx.out();
        out.push_str(&format!(
            r##"<use href="#{id}" transform="rotate({angle})"/>"##
        ));
        if mirror {
            out.push_str(&format!(
                r##"<use href="#{id}" transform="rotate({angle}) scale(-1 1)"/>"##
            ));
        }
    }
}

/// A smooth asymmetric closed curve occupying the wedge that points "up"
/// (towards -y) from the origin.
fn petal(prng: &mut Prng, length: f32, base_offset: f32, width_ratio: f32) -> String {
    let n = 8;
    let cy = -(base_offset + length * 0.5);
    let rx = length * width_ratio;
    let ry = length * 0.5;
    let skew = prng.range_f32(-0.35, 0.35);
    let bulge = prng.range_f32(0.1, 0.4);
    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        let theta = TAU * i as f32 / n as f32;
        let wobble = 1.0 + bulge * prng.range_f32(-1.0, 1.0);
        let x = rx * theta.cos() * wobble;
        let y = cy + ry * theta.sin() * wobble;
        // Shear along the petal axis so the shape is asymmetric.
        pts.push((x + skew * (y - cy), y));
    }
    let mut d = String::with_capacity(256);
    smooth_closed_d(&mut d, &pts);
    d
}

fn fmt(v: f32) -> String {
    let mut s = String::new();
    write_f32(&mut s, v);
    s
}
