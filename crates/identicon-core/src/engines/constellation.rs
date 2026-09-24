//! Constellation: a handful of stars linked by a hash-selected topology, with
//! one principal star. Star count, topology and the principal star's position
//! define the figure; dust and halos add depth.

use super::common::Ctx;
use crate::bezier::write_f32;
use crate::prng::Prng;

#[derive(Clone, Copy)]
enum Topology {
    Chain,
    Tree,
    Hub,
}

pub fn render(ctx: &mut Ctx, prng: &mut Prng) {
    let s = ctx.size;
    let c = ctx.center();

    let count = 5 + ctx.trait_u8(0, 5) as usize;
    let topology = match ctx.trait_u8(1, 3) {
        0 => Topology::Chain,
        1 => Topology::Tree,
        _ => Topology::Hub,
    };
    let principal = ctx.trait_u8(2, count as u8) as usize;
    let [ca, cb, cc] = Ctx::distinct3(u32::from(ctx.hash[3]) | (u32::from(ctx.hash[4]) << 8));

    let stars = place_stars(prng, count, s);
    let edges = link(&stars, topology, principal);

    let halo = ctx.id("halo");
    ctx.begin();
    {
        let color_a = ctx.color(ca);
        let out = ctx.out();
        out.push_str(&format!(
            r#"<radialGradient id="{halo}"><stop offset="0" stop-color="{color_a}" stop-opacity="0.85"/><stop offset="0.35" stop-color="{color_a}" stop-opacity="0.3"/><stop offset="1" stop-color="{color_a}" stop-opacity="0"/></radialGradient>"#
        ));
    }
    ctx.close_defs();

    let color_a = ctx.color(ca);
    let color_b = ctx.color(cb);
    let color_c = ctx.color(cc);

    // Background dust.
    for _ in 0..14 {
        let x = prng.range_f32(0.03, 0.97) * s;
        let y = prng.range_f32(0.03, 0.97) * s;
        let r = prng.range_f32(0.6, 1.6) * s / 256.0;
        ctx.write_circle(x, y, r.max(0.4), color_c, prng.range_f32(0.15, 0.4));
    }
    // A faint orbit ring gives the composition a frame.
    let orbit_r = s * (0.3 + 0.12 * ctx.trait_u8(5, 3) as f32);
    ctx.write_ring(c, c, orbit_r, color_b, s * 0.003, 0.28);

    for (a, b) in &edges {
        ctx.write_line(stars[*a], stars[*b], color_b, s * 0.0055, 0.6);
    }

    for (i, (x, y)) in stars.iter().enumerate() {
        if i == principal {
            continue;
        }
        let r = s * prng.range_f32(0.011, 0.02);
        ctx.write_circle(*x, *y, r * 2.2, color_c, 0.18);
        ctx.write_circle(*x, *y, r, if i % 3 == 0 { color_c } else { color_a }, 1.0);
    }

    let (px, py) = stars[principal];
    let pr = s * 0.032;
    ctx.write_circle(px, py, pr * 4.2, &format!("url(#{halo})"), 1.0);
    ctx.out().push_str(&format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="{color_a}""#,
        fmt(px),
        fmt(py),
        fmt(pr)
    ));
    if ctx.animated {
        ctx.out().push_str(&format!(
            r#"><animate attributeName="r" values="{};{};{}" dur="3.5s" repeatCount="indefinite"/></circle>"#,
            fmt(pr),
            fmt(pr * 1.25),
            fmt(pr)
        ));
    } else {
        ctx.out().push_str("/>");
    }
    ctx.write_circle(px, py, pr * 0.45, "#ffffff", 0.9);
}

/// Poisson-like placement: rejection sampling keeps stars apart so the figure
/// reads clearly at small sizes.
fn place_stars(prng: &mut Prng, count: usize, s: f32) -> Vec<(f32, f32)> {
    let mut stars: Vec<(f32, f32)> = Vec::with_capacity(count);
    let min_d = s * 0.17;
    for _ in 0..count {
        let mut best = (0.0, 0.0);
        let mut best_d = -1.0f32;
        for _ in 0..12 {
            let p = (
                prng.range_f32(0.14, 0.86) * s,
                prng.range_f32(0.14, 0.86) * s,
            );
            let d = stars
                .iter()
                .map(|q| ((q.0 - p.0).powi(2) + (q.1 - p.1).powi(2)).sqrt())
                .fold(f32::MAX, f32::min);
            if d > best_d {
                best_d = d;
                best = p;
            }
            if d >= min_d {
                break;
            }
        }
        stars.push(best);
    }
    stars
}

fn link(stars: &[(f32, f32)], topology: Topology, principal: usize) -> Vec<(usize, usize)> {
    let n = stars.len();
    let dist =
        |a: usize, b: usize| (stars[a].0 - stars[b].0).powi(2) + (stars[a].1 - stars[b].1).powi(2);
    match topology {
        Topology::Hub => (0..n)
            .filter(|&i| i != principal)
            .map(|i| (principal, i))
            .collect(),
        Topology::Tree => {
            // Prim's MST starting from the principal star.
            let mut in_tree = vec![false; n];
            in_tree[principal] = true;
            let mut edges = Vec::with_capacity(n - 1);
            for _ in 1..n {
                let mut best = (f32::MAX, 0, 0);
                for a in (0..n).filter(|&a| in_tree[a]) {
                    for b in (0..n).filter(|&b| !in_tree[b]) {
                        let d = dist(a, b);
                        if d < best.0 {
                            best = (d, a, b);
                        }
                    }
                }
                in_tree[best.2] = true;
                edges.push((best.1, best.2));
            }
            edges
        }
        Topology::Chain => {
            // Nearest-neighbour walk from the principal star.
            let mut visited = vec![false; n];
            visited[principal] = true;
            let mut current = principal;
            let mut edges = Vec::with_capacity(n - 1);
            for _ in 1..n {
                let next = (0..n)
                    .filter(|&b| !visited[b])
                    .min_by(|&a, &b| dist(current, a).partial_cmp(&dist(current, b)).unwrap())
                    .unwrap();
                visited[next] = true;
                edges.push((current, next));
                current = next;
            }
            edges
        }
    }
}

fn fmt(v: f32) -> String {
    let mut s = String::new();
    write_f32(&mut s, v);
    s
}
