//! Minimal geometric stroke alphabet on a 4 x 6 grid (x in 0..=4, y in 0..=6,
//! y grows downwards). Glyphs are SVG path fragments meant to be drawn with a
//! thick round-capped stroke, so the result is portable and font-independent.

pub const GLYPH_W: f32 = 4.0;
pub const GLYPH_H: f32 = 6.0;

pub fn glyph_path(ch: char) -> Option<&'static str> {
    Some(match ch.to_ascii_uppercase() {
        'A' => "M0,6 L2,0 L4,6 M0.8,3.8 L3.2,3.8",
        'B' => "M0,0 L0,6 M0,0 L2.5,0 Q4,0 4,1.5 Q4,3 2.5,3 L0,3 M2.5,3 Q4,3 4,4.5 Q4,6 2.5,6 L0,6",
        'C' => "M4,1.2 Q4,0 2.8,0 L1.2,0 Q0,0 0,1.2 L0,4.8 Q0,6 1.2,6 L2.8,6 Q4,6 4,4.8",
        'D' => "M0,0 L0,6 L2,6 Q4,6 4,4 L4,2 Q4,0 2,0 L0,0",
        'E' => "M4,0 L0,0 L0,6 L4,6 M0,3 L3,3",
        'F' => "M4,0 L0,0 L0,6 M0,3 L3,3",
        'G' => {
            "M4,1.2 Q4,0 2.8,0 L1.2,0 Q0,0 0,1.2 L0,4.8 Q0,6 1.2,6 L2.8,6 Q4,6 4,4.8 L4,3.2 L2.2,3.2"
        }
        'H' => "M0,0 L0,6 M4,0 L4,6 M0,3 L4,3",
        'I' => "M2,0 L2,6 M0.8,0 L3.2,0 M0.8,6 L3.2,6",
        'J' => "M4,0 L4,4.8 Q4,6 2.8,6 L1.2,6 Q0,6 0,4.8 L0,4",
        'K' => "M0,0 L0,6 M4,0 L0,3.6 M1.4,2.5 L4,6",
        'L' => "M0,0 L0,6 L4,6",
        'M' => "M0,6 L0,0 L2,3.2 L4,0 L4,6",
        'N' => "M0,6 L0,0 L4,6 L4,0",
        'O' => "M2,0 Q0,0 0,2 L0,4 Q0,6 2,6 Q4,6 4,4 L4,2 Q4,0 2,0",
        'P' => "M0,6 L0,0 L2.5,0 Q4,0 4,1.75 Q4,3.5 2.5,3.5 L0,3.5",
        'Q' => "M2,0 Q0,0 0,2 L0,4 Q0,6 2,6 Q4,6 4,4 L4,2 Q4,0 2,0 M2.6,4.4 L4,6",
        'R' => "M0,6 L0,0 L2.5,0 Q4,0 4,1.75 Q4,3.5 2.5,3.5 L0,3.5 M2,3.5 L4,6",
        'S' => {
            "M4,1.2 Q4,0 2.8,0 L1.2,0 Q0,0 0,1.5 Q0,3 1.5,3 L2.5,3 Q4,3 4,4.5 Q4,6 2.8,6 L1.2,6 Q0,6 0,4.8"
        }
        'T' => "M0,0 L4,0 M2,0 L2,6",
        'U' => "M0,0 L0,4 Q0,6 2,6 Q4,6 4,4 L4,0",
        'V' => "M0,0 L2,6 L4,0",
        'W' => "M0,0 L1,6 L2,2.6 L3,6 L4,0",
        'X' => "M0,0 L4,6 M4,0 L0,6",
        'Y' => "M0,0 L2,3 L4,0 M2,3 L2,6",
        'Z' => "M0,0 L4,0 L0,6 L4,6",
        '0' => "M2,0 Q0,0 0,2 L0,4 Q0,6 2,6 Q4,6 4,4 L4,2 Q4,0 2,0 M1,4.6 L3,1.4",
        '1' => "M0.8,1.2 L2,0 L2,6",
        '2' => "M0,1.5 Q0,0 2,0 Q4,0 4,1.5 Q4,3 2,4 L0,6 L4,6",
        '3' => "M0,0 L4,0 L2,2.5 Q4,2.5 4,4.25 Q4,6 2,6 Q0,6 0,5",
        '4' => "M3,6 L3,0 L0,4 L4,4",
        '5' => "M4,0 L0,0 L0,2.6 L2.5,2.6 Q4,2.6 4,4.3 Q4,6 2.5,6 Q1,6 0,5",
        '6' => "M3.4,0 L1.2,2.8 Q0,3.8 0,4.5 Q0,6 2,6 Q4,6 4,4.5 Q4,3 2,3 Q0,3 0,4.5",
        '7' => "M0,0 L4,0 L1.5,6",
        '8' => {
            "M2,3 Q0,3 0,1.5 Q0,0 2,0 Q4,0 4,1.5 Q4,3 2,3 Q0,3 0,4.5 Q0,6 2,6 Q4,6 4,4.5 Q4,3 2,3"
        }
        '9' => "M0.6,6 L2.8,3.2 Q4,2.2 4,1.5 Q4,0 2,0 Q0,0 0,1.5 Q0,3 2,3 Q4,3 4,1.5",
        _ => return None,
    })
}

/// Rewrites a grid-space glyph path into user space: scale by `k`, translate
/// by (`ox`, `oy`).
pub fn transform_glyph(src: &str, k: f32, ox: f32, oy: f32, out: &mut String) {
    for (i, token) in src.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let pair = if token.starts_with(|c: char| c.is_ascii_alphabetic()) {
            let (cmd, rest) = token.split_at(1);
            out.push_str(cmd);
            rest
        } else {
            token
        };
        let mut it = pair.split(',');
        let x: f32 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let y: f32 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        crate::bezier::write_f32(out, ox + x * k);
        out.push(',');
        crate::bezier::write_f32(out, oy + y * k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_alnum_glyphs_exist() {
        for ch in ('A'..='Z').chain('0'..='9') {
            assert!(glyph_path(ch).is_some(), "{ch}");
        }
        assert!(glyph_path('-').is_none());
    }

    #[test]
    fn transform_scales_and_offsets() {
        let mut out = String::new();
        transform_glyph("M0,0 L4,6", 10.0, 5.0, 7.0, &mut out);
        assert_eq!(out, "M5,7 L45,67");
    }
}
