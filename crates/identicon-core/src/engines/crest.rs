//! Heraldic crest: a contour (escutcheon), a field partition in two colours,
//! and a simple central charge. Three macroscopic traits drive recognisability.

use super::common::{Contour, Ctx, polar, star_polygon, write_pt};
use crate::bezier::write_f32;
use crate::prng::Prng;
use std::f32::consts::TAU;

#[derive(Clone, Copy)]
enum Partition {
    Parti,
    Coupe,
    Ecartele,
    Chevron,
    Bande,
    Pal,
}

#[derive(Clone, Copy)]
enum Charge {
    Star,
    Crescent,
    Lozenge,
    Ring,
    Chevron,
    Roundel,
}

pub fn render(ctx: &mut Ctx, prng: &mut Prng) {
    let s = ctx.size;
    let c = ctx.center();
    let r = s * 0.38;

    let contour = Contour::from_index(ctx.trait_u8(0, 5));
    let partition = match ctx.trait_u8(1, 6) {
        0 => Partition::Parti,
        1 => Partition::Coupe,
        2 => Partition::Ecartele,
        3 => Partition::Chevron,
        4 => Partition::Bande,
        _ => Partition::Pal,
    };
    let charge = match ctx.trait_u8(2, 6) {
        0 => Charge::Star,
        1 => Charge::Crescent,
        2 => Charge::Lozenge,
        3 => Charge::Ring,
        4 => Charge::Chevron,
        _ => Charge::Roundel,
    };
    let [ca, cb, cc] = Ctx::distinct3(u32::from(ctx.hash[3]) | (u32::from(ctx.hash[4]) << 8));
    let triple = ctx.trait_u8(5, 3) == 0;
    let rotation = prng.range_f32(-0.12, 0.12);

    let clip = ctx.id("clip");
    let shade = ctx.id("shade");
    let contour_d = contour.path_d(c, c, r);

    ctx.begin();
    {
        let out = ctx.out();
        out.push_str(&format!(
            r#"<clipPath id="{clip}"><path d="{contour_d}"/></clipPath>"#
        ));
        out.push_str(&format!(
            r##"<radialGradient id="{shade}" cx="0.5" cy="0.35" r="0.75"><stop offset="0" stop-color="#ffffff" stop-opacity="0.16"/><stop offset="0.6" stop-color="#ffffff" stop-opacity="0"/><stop offset="1" stop-color="#000000" stop-opacity="0.28"/></radialGradient>"##
        ));
    }
    ctx.close_defs();

    let color_a = ctx.color(ca);
    let color_b = ctx.color(cb);
    let color_c = ctx.color(cc);
    let bg = ctx.palette.bg_dark;

    // Drop shadow under the escutcheon.
    ctx.write_path(
        &contour.path_d(c, c + s * 0.012, r * 1.01),
        &format!(r#"fill="{bg}" fill-opacity="0.45""#),
    );

    ctx.out()
        .push_str(&format!(r#"<g clip-path="url(#{clip})">"#));
    // Field.
    let full = (c - r * 1.2, c - r * 1.2, r * 2.4);
    let mut rect_d = String::new();
    rect_d.push('M');
    write_pt(&mut rect_d, (full.0, full.1));
    rect_d.push('h');
    write_f32(&mut rect_d, full.2);
    rect_d.push('v');
    write_f32(&mut rect_d, full.2);
    rect_d.push('h');
    write_f32(&mut rect_d, -full.2);
    rect_d.push('Z');
    ctx.write_path(&rect_d, &format!(r#"fill="{color_a}""#));

    let (x0, y0, x1, y1) = (c - r * 1.2, c - r * 1.2, c + r * 1.2, c + r * 1.2);
    let attrs_b = format!(r#"fill="{color_b}""#);
    match partition {
        Partition::Parti => ctx.write_polygon(&[(c, y0), (x1, y0), (x1, y1), (c, y1)], &attrs_b),
        Partition::Coupe => ctx.write_polygon(&[(x0, c), (x1, c), (x1, y1), (x0, y1)], &attrs_b),
        Partition::Ecartele => {
            ctx.write_polygon(&[(c, y0), (x1, y0), (x1, c), (c, c)], &attrs_b);
            ctx.write_polygon(&[(x0, c), (c, c), (c, y1), (x0, y1)], &attrs_b);
        }
        Partition::Chevron => {
            let apex = c - r * 0.35;
            ctx.write_polygon(
                &[
                    (x0, y1),
                    (x0, c + r * 0.55),
                    (c, apex),
                    (x1, c + r * 0.55),
                    (x1, y1),
                ],
                &attrs_b,
            );
        }
        Partition::Bande => {
            let w = r * 0.5;
            ctx.write_polygon(
                &[(x0, y0 + w), (x0 + w, y0), (x1, y1 - w), (x1 - w, y1)],
                &format!(
                    r#"fill="{color_b}" transform="rotate({} {c} {c})""#,
                    rotation.to_degrees() * 4.0
                ),
            );
        }
        Partition::Pal => {
            let w = r * 0.42;
            ctx.write_polygon(
                &[
                    (c - w * 1.5, y0),
                    (c - w * 0.5, y0),
                    (c - w * 0.5, y1),
                    (c - w * 1.5, y1),
                ],
                &attrs_b,
            );
            ctx.write_polygon(
                &[
                    (c + w * 0.5, y0),
                    (c + w * 1.5, y0),
                    (c + w * 1.5, y1),
                    (c + w * 0.5, y1),
                ],
                &attrs_b,
            );
        }
    }

    // Charge(s).
    let charge_r = if triple { r * 0.22 } else { r * 0.36 };
    let positions: Vec<(f32, f32)> = if triple {
        vec![
            (c, c - r * 0.42),
            (c - r * 0.42, c + r * 0.3),
            (c + r * 0.42, c + r * 0.3),
        ]
    } else {
        vec![(c, c)]
    };
    let anim = ctx.animated;
    for (i, (px, py)) in positions.iter().enumerate() {
        write_charge(
            ctx,
            charge,
            *px,
            *py,
            charge_r,
            rotation,
            color_c,
            bg,
            anim && i == 0,
        );
    }

    // Glossy vignette over the whole field.
    ctx.write_path(&contour_d, &format!(r#"fill="url(#{shade})""#));
    ctx.out().push_str("</g>");

    // Rim.
    ctx.write_path(
        &contour_d,
        &format!(
            r#"fill="none" stroke="{color_c}" stroke-opacity="0.85" stroke-width="{}""#,
            fmt(s * 0.012)
        ),
    );
    ctx.write_path(
        &contour.path_d(c, c, r * 0.9),
        &format!(
            r#"fill="none" stroke="{bg}" stroke-opacity="0.35" stroke-width="{}""#,
            fmt(s * 0.006)
        ),
    );
}

#[allow(clippy::too_many_arguments)]
fn write_charge(
    ctx: &mut Ctx,
    charge: Charge,
    cx: f32,
    cy: f32,
    r: f32,
    rotation: f32,
    color: &str,
    outline: &str,
    animate: bool,
) {
    let stroke = format!(
        r#"stroke="{outline}" stroke-opacity="0.55" stroke-width="{}" stroke-linejoin="round""#,
        fmt(r * 0.08)
    );
    let attrs = format!(r#"fill="{color}" {stroke}"#);

    if animate {
        ctx.out().push_str(r#"<g opacity="0.9">"#);
    }
    match charge {
        Charge::Star => {
            let pts = star_polygon(cx, cy, r, r * 0.45, 5, -TAU / 4.0 + rotation);
            ctx.write_polygon(&pts, &attrs);
        }
        Charge::Lozenge => {
            let pts = [
                (cx, cy - r),
                (cx + r * 0.7, cy),
                (cx, cy + r),
                (cx - r * 0.7, cy),
            ];
            ctx.write_polygon(&pts, &attrs);
        }
        Charge::Roundel => {
            ctx.write_circle(cx, cy, r * 0.9, color, 1.0);
            ctx.write_ring(cx, cy, r * 0.9, outline, r * 0.08, 0.55);
        }
        Charge::Ring => {
            ctx.write_ring(cx, cy, r * 0.75, color, r * 0.32, 1.0);
        }
        Charge::Chevron => {
            let w = r * 0.32;
            let pts = [
                (cx - r, cy + r * 0.6),
                (cx, cy - r * 0.6),
                (cx + r, cy + r * 0.6),
                (cx + r - w, cy + r * 0.6),
                (cx, cy - r * 0.6 + w * 1.4),
                (cx - r + w, cy + r * 0.6),
            ];
            ctx.write_polygon(&pts, &attrs);
        }
        Charge::Crescent => {
            let mut d = String::with_capacity(96);
            let a0 = -TAU * 0.3 + rotation;
            let a1 = TAU * 0.3 + rotation;
            let p0 = polar(cx, cy, r, a0);
            let p1 = polar(cx, cy, r, a1);
            d.push('M');
            write_pt(&mut d, p0);
            d.push('A');
            write_pt(&mut d, (r, r));
            d.push_str(" 0 1 1 ");
            write_pt(&mut d, p1);
            d.push('A');
            write_pt(&mut d, (r * 0.78, r * 0.78));
            d.push_str(" 0 1 0 ");
            write_pt(&mut d, p0);
            d.push('Z');
            ctx.write_path(&d, &attrs);
        }
    }
    if animate {
        ctx.out().push_str(
            r#"<animate attributeName="opacity" values="0.8;1;0.8" dur="4s" repeatCount="indefinite"/></g>"#,
        );
    }
}

fn fmt(v: f32) -> String {
    let mut s = String::new();
    write_f32(&mut s, v);
    s
}
