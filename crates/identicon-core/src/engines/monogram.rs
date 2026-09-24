//! Generative monogram: initials extracted from the input, drawn with the
//! built-in geometric stroke alphabet over a hash-selected motif. Inputs
//! without any letter fall back to the crest engine.

use super::common::{Contour, Ctx, polygon_d};
use super::glyphs::{GLYPH_H, GLYPH_W, glyph_path, transform_glyph};
use crate::bezier::write_f32;
use crate::prng::Prng;

#[derive(Clone, Copy)]
enum Motif {
    Stripes,
    Rings,
    Disk,
    Split,
}

pub fn render(ctx: &mut Ctx, prng: &mut Prng, input: &str) {
    let Some(initials) = initials(input) else {
        super::crest::render(ctx, prng);
        return;
    };

    let s = ctx.size;
    let c = ctx.center();
    let r = s * 0.42;

    let motif = match ctx.trait_u8(0, 4) {
        0 => Motif::Stripes,
        1 => Motif::Rings,
        2 => Motif::Disk,
        _ => Motif::Split,
    };
    let contour = if ctx.trait_u8(1, 2) == 0 {
        Contour::Circle
    } else {
        Contour::RoundedSquare
    };
    let [ca, cb, cc] = Ctx::distinct3(u32::from(ctx.hash[2]) | (u32::from(ctx.hash[3]) << 8));
    let angle = prng.range_f32(-40.0, 40.0);
    let offset = (prng.range_f32(-0.2, 0.2) * s, prng.range_f32(-0.2, 0.2) * s);

    let clip = ctx.id("clip");
    let contour_d = contour.path_d(c, c, r);

    ctx.begin();
    ctx.out().push_str(&format!(
        r#"<clipPath id="{clip}"><path d="{contour_d}"/></clipPath>"#
    ));
    ctx.close_defs();

    let color_a = ctx.color(ca);
    let color_b = ctx.color(cb);
    let color_c = ctx.color(cc);
    let bg = ctx.palette.bg_dark;

    // Plate.
    ctx.write_path(
        &contour.path_d(c, c + s * 0.01, r * 1.01),
        &format!(r#"fill="{bg}" fill-opacity="0.5""#),
    );
    ctx.write_path(
        &contour_d,
        &format!(r#"fill="{color_a}" fill-opacity="0.16""#),
    );

    // Motif.
    ctx.out()
        .push_str(&format!(r#"<g clip-path="url(#{clip})">"#));
    let anim = ctx.animated;
    match motif {
        Motif::Stripes => {
            let w = s * 0.07;
            ctx.out().push_str(&format!(
                r#"<g transform="rotate({} {} {})">"#,
                fmt(angle),
                fmt(c),
                fmt(c)
            ));
            for i in -3..=3 {
                let x = c + i as f32 * w * 2.0;
                let mut d = String::new();
                polygon_d(
                    &mut d,
                    &[(x, -s), (x + w, -s), (x + w, 2.0 * s), (x, 2.0 * s)],
                );
                ctx.write_path(&d, &format!(r#"fill="{color_b}" fill-opacity="0.32""#));
            }
            ctx.out().push_str("</g>");
        }
        Motif::Rings => {
            let (ox, oy) = (c + offset.0, c + offset.1);
            for i in 1..=4 {
                ctx.write_ring(ox, oy, s * 0.11 * i as f32, color_b, s * 0.028, 0.3);
            }
        }
        Motif::Disk => {
            ctx.write_circle(
                c + offset.0 * 1.6,
                c + offset.1 * 1.6,
                s * 0.3,
                color_b,
                0.5,
            );
        }
        Motif::Split => {
            let mut d = String::new();
            polygon_d(
                &mut d,
                &[(-s, c), (2.0 * s, c), (2.0 * s, 2.0 * s), (-s, 2.0 * s)],
            );
            ctx.write_path(
                &d,
                &format!(
                    r#"fill="{color_b}" fill-opacity="0.45" transform="rotate({} {} {})""#,
                    fmt(angle),
                    fmt(c),
                    fmt(c)
                ),
            );
        }
    }
    if anim {
        ctx.out().push_str(
            r#"<animate attributeName="opacity" values="1;0.6;1" dur="6s" repeatCount="indefinite"/>"#,
        );
    }
    ctx.out().push_str("</g>");

    // Letters.
    let glyph_h = s * (if initials.len() == 1 { 0.46 } else { 0.4 });
    let k = glyph_h / GLYPH_H;
    let gap = k * 1.4;
    let total_w = GLYPH_W * k * initials.len() as f32 + gap * (initials.len() as f32 - 1.0);
    let x0 = c - total_w * 0.5;
    let y0 = c - glyph_h * 0.5;
    let stroke_w = fmt(k * 0.9);

    let mut d = String::with_capacity(256);
    for (i, ch) in initials.iter().enumerate() {
        if let Some(src) = glyph_path(*ch) {
            if !d.is_empty() {
                d.push(' ');
            }
            transform_glyph(src, k, x0 + i as f32 * (GLYPH_W * k + gap), y0, &mut d);
        }
    }
    let shadow = fmt(s * 0.018);
    ctx.write_path(
        &d,
        &format!(
            r#"fill="none" stroke="{color_a}" stroke-opacity="0.75" stroke-width="{stroke_w}" stroke-linecap="round" stroke-linejoin="round" transform="translate({shadow} {shadow})""#
        ),
    );
    ctx.write_path(
        &d,
        &format!(
            r#"fill="none" stroke="{color_c}" stroke-width="{stroke_w}" stroke-linecap="round" stroke-linejoin="round""#
        ),
    );

    ctx.write_path(
        &contour_d,
        &format!(
            r#"fill="none" stroke="{color_a}" stroke-opacity="0.8" stroke-width="{}""#,
            fmt(s * 0.01)
        ),
    );
}

/// Up to two glyph characters: first letters of the first two name tokens, or
/// the first two characters of a single token. `None` when the input has no
/// ASCII letter at all (e.g. numeric ids).
pub fn initials(input: &str) -> Option<Vec<char>> {
    let local = input.split('@').next().unwrap_or(input);
    let tokens: Vec<&str> = local
        .split(['.', '-', '_'])
        .filter(|t| t.chars().any(|c| c.is_ascii_alphanumeric()))
        .collect();
    if !local.chars().any(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let first_alnum = |t: &str| {
        t.chars()
            .find(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_uppercase())
    };
    let mut out = Vec::with_capacity(2);
    match tokens.as_slice() {
        [] => return None,
        [single] => {
            let mut chars = single
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .map(|c| c.to_ascii_uppercase());
            out.extend(chars.next());
            out.extend(chars.next());
        }
        [a, b, ..] => {
            out.extend(first_alnum(a));
            out.extend(first_alnum(b));
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

fn fmt(v: f32) -> String {
    let mut s = String::new();
    write_f32(&mut s, v);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_from_tokens() {
        assert_eq!(initials("alice"), Some(vec!['A', 'L']));
        assert_eq!(initials("jean.dupont@example.org"), Some(vec!['J', 'D']));
        assert_eq!(initials("bob_martin"), Some(vec!['B', 'M']));
        assert_eq!(initials("x"), Some(vec!['X']));
        assert_eq!(initials("12345"), None);
    }
}
