use crate::bezier::{smooth_blob_path, write_path_d};
use crate::palette::Palette;
use crate::prng::Prng;
const MAX_POINTS: usize = 14;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug)]
pub struct BlobSpec {
    pub cx: f32,
    pub cy: f32,
    pub base_radius: f32,
    pub point_count: usize,
    pub color_indices: [usize; 3],
    pub opacity: f32,
    pub use_radial: bool,
    pub z: u32,
}

#[derive(Clone, Debug)]
pub struct RenderedBlob {
    pub path_d: String,
    pub color_indices: [usize; 3],
    pub opacity: f32,
    pub use_radial: bool,
    pub cx: f32,
    pub cy: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symmetry {
    None,
    Mirror,
    Radial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundStyle {
    Light,
    Dark,
}

pub struct RenderParams {
    pub blob_count: u32,
    pub palette_index: usize,
    pub symmetry: Symmetry,
    pub background_style: BackgroundStyle,
    pub curve_strength: f32,
    pub gradient_angle: f32,
    pub blob_scale: f32,
    pub accent_count: u32,
}

impl RenderParams {
    pub fn from_prng(prng: &mut Prng) -> Self {
        Self {
            blob_count: prng.range_u32(3, 8),
            palette_index: prng.next_u32() as usize,
            symmetry: match prng.range_u32(0, 2) {
                0 => Symmetry::None,
                1 => Symmetry::Mirror,
                _ => Symmetry::Radial,
            },
            background_style: if prng.next_f32() > 0.5 {
                BackgroundStyle::Light
            } else {
                BackgroundStyle::Dark
            },
            curve_strength: prng.range_f32(0.2, 0.8),
            gradient_angle: prng.range_f32(0.0, 360.0),
            blob_scale: prng.range_f32(0.5, 1.5),
            accent_count: prng.range_u32(2, 6),
        }
    }
}

pub fn generate_blob_specs(prng: &mut Prng, params: &RenderParams) -> Vec<BlobSpec> {
    let mut specs = Vec::with_capacity(params.blob_count as usize * 4);

    for i in 0..params.blob_count {
        let cx = prng.range_f32(0.2, 0.8);
        let cy = prng.range_f32(0.2, 0.8);
        let base_radius = prng.range_f32(0.12, 0.28) * params.blob_scale;
        let point_count = prng.range_u32(8, 10) as usize;
        let palette_len = 5usize;
        let color_indices = [
            (prng.next_u32() as usize) % palette_len,
            (prng.next_u32() as usize) % palette_len,
            (prng.next_u32() as usize) % palette_len,
        ];
        let opacity = prng.range_f32(0.55, 0.92);
        let use_radial = i % 2 == 0;
        let z = prng.next_u32();

        let base = BlobSpec {
            cx,
            cy,
            base_radius,
            point_count,
            color_indices,
            opacity,
            use_radial,
            z,
        };

        match params.symmetry {
            Symmetry::None => specs.push(base),
            Symmetry::Mirror => {
                specs.push(base.clone());
                specs.push(BlobSpec {
                    cx: 1.0 - cx,
                    ..base
                });
            }
            Symmetry::Radial => {
                let copies = prng.range_u32(2, 4);
                for k in 0..copies {
                    let angle = (k as f32 / copies as f32) * std::f32::consts::TAU;
                    let dx = cx - 0.5;
                    let dy = cy - 0.5;
                    let rot_x = 0.5 + dx * angle.cos() - dy * angle.sin();
                    let rot_y = 0.5 + dx * angle.sin() + dy * angle.cos();
                    specs.push(BlobSpec {
                        cx: rot_x.clamp(0.15, 0.85),
                        cy: rot_y.clamp(0.15, 0.85),
                        ..base
                    });
                }
            }
        }
    }

    specs.sort_by_key(|s| s.z);
    specs
}

pub fn render_blobs(
    prng: &mut Prng,
    specs: &[BlobSpec],
    params: &RenderParams,
    size: u32,
) -> Vec<RenderedBlob> {
    let scale = size as f32;

    specs
        .iter()
        .map(|spec| {
            let mut perturbations = [0.0f32; MAX_POINTS];
            for p in perturbations.iter_mut().take(spec.point_count) {
                *p = prng.range_f32(-1.0, 1.0);
            }

            let cx = spec.cx * scale;
            let cy = spec.cy * scale;
            let base_radius = spec.base_radius * scale;

            let points = smooth_blob_path(
                cx,
                cy,
                base_radius,
                spec.point_count,
                params.curve_strength,
                &perturbations,
            );

            let mut path_d = String::with_capacity(256);
            write_path_d(&mut path_d, &points);

            RenderedBlob {
                path_d,
                color_indices: spec.color_indices,
                opacity: spec.opacity,
                use_radial: spec.use_radial,
                cx,
                cy,
            }
        })
        .collect()
}

pub struct Accent {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub color_index: usize,
    pub opacity: f32,
}

pub fn generate_accents(prng: &mut Prng, params: &RenderParams, size: u32) -> Vec<Accent> {
    let scale = size as f32;
    (0..params.accent_count)
        .map(|_| Accent {
            x: prng.range_f32(0.05, 0.95) * scale,
            y: prng.range_f32(0.05, 0.95) * scale,
            r: prng.range_f32(2.0, 6.0),
            color_index: (prng.next_u32() as usize) % 5,
            opacity: prng.range_f32(0.25, 0.55),
        })
        .collect()
}

pub fn palette_color(palette: &Palette, index: usize) -> &'static str {
    palette.colors[index % palette.colors.len()]
}
