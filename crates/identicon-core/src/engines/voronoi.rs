//! Stained-glass Voronoi tessellation with a hash-selected symmetry.
//! The symmetry group and the contour drive the silhouette; the cell layout
//! and colours carry the fine detail.

use super::common::{Contour, Ctx, polygon_d};
use crate::bezier::write_f32;
use crate::prng::Prng;
use std::f32::consts::TAU;

#[derive(Clone, Copy)]
enum Symmetry {
    MirrorX,
    MirrorXY,
    Rot3,
    Rot4,
}

struct Site {
    x: f32,
    y: f32,
    color: usize,
}

pub fn render(ctx: &mut Ctx, prng: &mut Prng) {
    let s = ctx.size;
    let c = ctx.center();
    let r = s * 0.42;

    let symmetry = match ctx.trait_u8(0, 4) {
        0 => Symmetry::MirrorX,
        1 => Symmetry::MirrorXY,
        2 => Symmetry::Rot3,
        _ => Symmetry::Rot4,
    };
    let contour = if ctx.trait_u8(1, 2) == 0 {
        Contour::Circle
    } else {
        Contour::RoundedSquare
    };
    let base_count = 5 + ctx.trait_u8(2, 4) as usize;
    let dominant = ctx.trait_u8(3, 5) as usize;

    let sites = generate_sites(prng, symmetry, base_count, dominant);

    let clip = ctx.id("clip");
    let gloss = ctx.id("gloss");
    let contour_d = contour.path_d(c, c, r);

    ctx.begin();
    {
        let animated = ctx.animated;
        let out = ctx.out();
        out.push_str(&format!(
            r#"<clipPath id="{clip}"><path d="{contour_d}"/></clipPath>"#
        ));
        out.push_str(&format!(
            r##"<linearGradient id="{gloss}" x1="0" y1="0" x2="0.6" y2="1"><stop offset="0" stop-color="#ffffff" stop-opacity="0.28"/><stop offset="0.5" stop-color="#ffffff" stop-opacity="0.02"/><stop offset="1" stop-color="#000000" stop-opacity="0.3"/>"##
        ));
        if animated {
            out.push_str(r#"<animate attributeName="x2" values="0.6;1;0.6" dur="7s" repeatCount="indefinite"/>"#);
        }
        out.push_str("</linearGradient>");
    }
    ctx.close_defs();

    let bg = ctx.palette.bg_dark;
    let lead = fmt(s * 0.014);

    ctx.write_path(
        &contour.path_d(c, c + s * 0.01, r * 1.01),
        &format!(r#"fill="{bg}" fill-opacity="0.5""#),
    );
    ctx.out().push_str(&format!(r#"<g clip-path="url(#{clip})" stroke="{bg}" stroke-width="{lead}" stroke-linejoin="round">"#));

    let span = 2.4 * r;
    let origin = c - 1.2 * r;
    for (i, site) in sites.iter().enumerate() {
        let cell = voronoi_cell(&sites, i);
        if cell.len() < 3 {
            continue;
        }
        let pts: Vec<(f32, f32)> = cell
            .iter()
            .map(|(x, y)| (origin + x * span, origin + y * span))
            .collect();
        let color = ctx.color(site.color);
        let mut d = String::with_capacity(pts.len() * 12);
        polygon_d(&mut d, &pts);
        ctx.write_path(&d, &format!(r#"fill="{color}" fill-opacity="0.92""#));
    }

    ctx.write_path(
        &contour_d,
        &format!(r#"fill="url(#{gloss})" stroke="none""#),
    );
    ctx.out().push_str("</g>");

    let rim = ctx.color(dominant);
    ctx.write_path(
        &contour_d,
        &format!(
            r#"fill="none" stroke="{rim}" stroke-opacity="0.7" stroke-width="{}""#,
            fmt(s * 0.01)
        ),
    );
}

fn generate_sites(
    prng: &mut Prng,
    symmetry: Symmetry,
    base_count: usize,
    dominant: usize,
) -> Vec<Site> {
    let mut sites: Vec<Site> = Vec::with_capacity(base_count * 4);
    let push = |sites: &mut Vec<Site>, x: f32, y: f32, color: usize| {
        let x = x.clamp(0.02, 0.98);
        let y = y.clamp(0.02, 0.98);
        if sites
            .iter()
            .all(|s| (s.x - x).abs() + (s.y - y).abs() > 0.06)
        {
            sites.push(Site { x, y, color });
        }
    };

    for _ in 0..base_count {
        let x = prng.range_f32(0.05, 0.95);
        let y = prng.range_f32(0.05, 0.95);
        // Roughly half of the cells use the dominant colour so the identicon has
        // a clear hue even when the palette is loud.
        let color = if prng.next_f32() < 0.45 {
            dominant
        } else {
            (prng.next_u32() % 5) as usize
        };
        match symmetry {
            Symmetry::MirrorX => {
                push(&mut sites, x, y, color);
                push(&mut sites, 1.0 - x, y, color);
            }
            Symmetry::MirrorXY => {
                push(&mut sites, x, y, color);
                push(&mut sites, 1.0 - x, y, color);
                push(&mut sites, x, 1.0 - y, color);
                push(&mut sites, 1.0 - x, 1.0 - y, color);
            }
            Symmetry::Rot3 | Symmetry::Rot4 => {
                let n = if matches!(symmetry, Symmetry::Rot3) {
                    3
                } else {
                    4
                };
                let (dx, dy) = (x - 0.5, y - 0.5);
                for k in 0..n {
                    let a = TAU * k as f32 / n as f32;
                    let rx = 0.5 + dx * a.cos() - dy * a.sin();
                    let ry = 0.5 + dx * a.sin() + dy * a.cos();
                    push(&mut sites, rx, ry, color);
                }
            }
        }
    }
    sites
}

/// Voronoi cell of `sites[i]` in unit-square coordinates, obtained by clipping
/// a generous bounding polygon against every bisector half-plane.
fn voronoi_cell(sites: &[Site], i: usize) -> Vec<(f32, f32)> {
    let mut poly: Vec<(f32, f32)> = vec![(-0.6, -0.6), (1.6, -0.6), (1.6, 1.6), (-0.6, 1.6)];
    let (sx, sy) = (sites[i].x, sites[i].y);
    for (j, other) in sites.iter().enumerate() {
        if j == i {
            continue;
        }
        let (nx, ny) = (other.x - sx, other.y - sy);
        let (mx, my) = ((sx + other.x) * 0.5, (sy + other.y) * 0.5);
        let inside = |p: (f32, f32)| (p.0 - mx) * nx + (p.1 - my) * ny <= 0.0;
        let mut next = Vec::with_capacity(poly.len() + 2);
        for k in 0..poly.len() {
            let a = poly[k];
            let b = poly[(k + 1) % poly.len()];
            let (ia, ib) = (inside(a), inside(b));
            if ia {
                next.push(a);
            }
            if ia != ib {
                let da = (a.0 - mx) * nx + (a.1 - my) * ny;
                let db = (b.0 - mx) * nx + (b.1 - my) * ny;
                let t = da / (da - db);
                next.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        poly = next;
        if poly.len() < 3 {
            break;
        }
    }
    poly
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
    fn cells_tile_without_degenerate_polygons() {
        let sites = vec![
            Site {
                x: 0.2,
                y: 0.2,
                color: 0,
            },
            Site {
                x: 0.8,
                y: 0.2,
                color: 1,
            },
            Site {
                x: 0.5,
                y: 0.8,
                color: 2,
            },
        ];
        for i in 0..sites.len() {
            assert!(voronoi_cell(&sites, i).len() >= 3);
        }
    }
}
