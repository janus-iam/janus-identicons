use crate::blob::{generate_accents, generate_blob_specs, render_blobs, RenderParams};
use crate::hash::{hash_input, validate_input};
use crate::palette::{palette_by_index, palette_by_theme};
use crate::prng::Prng;
use crate::svg::{clamp_size, SvgBuilder};
use crate::{RenderError, RenderOptions};

pub fn render_identicon_inner(input: &str, opts: &RenderOptions) -> Result<String, RenderError> {
    validate_input(input)?;
    let hash = hash_input(input);
    let mut prng = Prng::from_seed(hash);

    let params = RenderParams::from_prng(&mut prng);

    let palette = if let Some(theme) = opts.theme {
        palette_by_theme(theme)
    } else {
        palette_by_index(params.palette_index)
    };

    let size = clamp_size(opts.size);
    let specs = generate_blob_specs(&mut prng, &params);
    let blobs = render_blobs(&mut prng, &specs, &params, size);
    let accents = generate_accents(&mut prng, &params, size);

    let anim_seed = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
    let mut builder = SvgBuilder::new(4096, opts.animated, anim_seed);
    builder.begin(size);

    if opts.background {
        builder.write_background_gradient(palette, params.background_style, params.gradient_angle);
    }

    for (i, blob) in blobs.iter().enumerate() {
        let id = grad_id_buf(i);
        builder.write_blob_gradient(&id, blob, palette, size);
    }

    builder.close_defs();

    if opts.background {
        builder.write_background_rect();
    }

    for (i, blob) in blobs.iter().enumerate() {
        builder.write_blob_path(&grad_id_buf(i), blob);
    }

    for (i, accent) in accents.iter().enumerate() {
        builder.write_accent(palette, accent, i);
    }

    builder.finish();
    Ok(builder.into_string())
}

fn grad_id_buf(i: usize) -> String {
    let mut s = String::with_capacity(4);
    s.push('g');
    let mut n = i;
    if n == 0 {
        s.push('0');
    } else {
        let mut digits = [0u8; 4];
        let mut len = 0;
        while n > 0 {
            digits[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
        for d in digits[..len].iter().rev() {
            s.push(*d as char);
        }
    }
    s
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
}
