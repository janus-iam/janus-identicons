use crate::hash::{hash_input, validate_input};
use crate::palette::{palette_by_index, palette_by_theme};
use crate::prng::Prng;
use crate::sigil::{generate_arc_layers, generate_orbit_nodes, SigilParams};
use crate::svg::{clamp_size, SvgBuilder};
use crate::{RenderError, RenderOptions};

pub fn render_identicon_inner(input: &str, opts: &RenderOptions) -> Result<String, RenderError> {
    validate_input(input)?;
    let hash = hash_input(input);
    let mut prng = Prng::from_seed(hash);

    let params = SigilParams::from_hash_and_prng(&hash, &mut prng);

    let palette = if let Some(theme) = opts.theme {
        palette_by_theme(theme)
    } else {
        palette_by_index(params.palette_index)
    };

    let size = clamp_size(opts.size);
    let size_f = size as f32;
    let cx = size_f * 0.5;
    let cy = size_f * 0.5;

    let arcs = generate_arc_layers(&mut prng, &params, size_f);
    let nodes = generate_orbit_nodes(&mut prng, &params, size_f);

    let anim_seed = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
    let mut builder = SvgBuilder::new(4096, opts.animated, anim_seed);
    builder.begin(size);

    if opts.background {
        builder.write_background(palette, params.background_style, size);
    }

    builder.close_defs();

    if opts.background {
        builder.write_background_rect();
    }

    let guide_color = palette.colors[params.palette_index % palette.colors.len()];
    builder.write_guide_rings(cx, cy, size_f, guide_color, 0.12);

    for arc in &arcs {
        builder.write_arc(cx, cy, arc, palette);
    }

    builder.begin_orbit_group(cx, cy, 0);
    for node in &nodes {
        builder.write_orbit_node(cx, cy, node, palette);
    }
    builder.end_group();

    let center_color = (hash[7] as usize) % palette.colors.len();
    builder.write_center(
        params.center_glyph,
        cx,
        cy,
        size_f,
        palette,
        center_color,
    );

    builder.finish();
    Ok(builder.into_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOptions, Theme};

    #[test]
    fn deterministic_output() {
        let a = render_identicon_inner("alice", &RenderOptions::default()).unwrap();
        let b = render_identicon_inner("alice", &RenderOptions::default()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn different_inputs_differ() {
        let a = render_identicon_inner("alice", &RenderOptions::default()).unwrap();
        let b = render_identicon_inner("bob", &RenderOptions::default()).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn under_10kb() {
        let svg = render_identicon_inner("test-user-123", &RenderOptions::default()).unwrap();
        assert!(svg.len() < 10 * 1024);
    }

    #[test]
    fn contains_sigil_elements() {
        let svg = render_identicon_inner("alice", &RenderOptions::default()).unwrap();
        assert!(svg.contains("<path"));
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn theme_override() {
        let opts = RenderOptions {
            theme: Some(Theme::Synthwave),
            ..RenderOptions::default()
        };
        let svg = render_identicon_inner("alice", &opts).unwrap();
        assert!(svg.contains("svg"));
    }

    #[test]
    fn animated_contains_animate() {
        let opts = RenderOptions {
            animated: true,
            ..RenderOptions::default()
        };
        let svg = render_identicon_inner("alice", &opts).unwrap();
        assert!(svg.contains("<animate"));
    }

    #[test]
    fn rejects_empty() {
        assert!(render_identicon_inner("", &RenderOptions::default()).is_err());
    }

    #[test]
    fn rejects_too_long() {
        let s = "a".repeat(300);
        assert!(render_identicon_inner(&s, &RenderOptions::default()).is_err());
    }

    #[test]
    fn svg_attributes_are_closed() {
        let opts = RenderOptions {
            size: 128,
            animated: true,
            ..RenderOptions::default()
        };
        let svg = render_identicon_inner("alice", &opts).unwrap();
        for tag in svg.split('<').skip(1) {
            let part = tag.split('>').next().unwrap_or(tag);
            let quotes = part.chars().filter(|c| *c == '"').count();
            assert_eq!(quotes % 2, 0, "unclosed quote in tag: <{part}");
        }
    }
}
