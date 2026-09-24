pub mod common;
pub mod constellation;
pub mod crest;
pub mod glyphs;
pub mod kaleido;
pub mod monogram;
pub mod ribbon;
pub mod voronoi;

use crate::prng::Prng;
pub use common::Ctx;

/// Visual engine used to turn a hash into shapes. Palettes/themes are shared
/// by every engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Engine {
    /// Original gradient-blob renderer.
    #[default]
    Blob,
    /// Heraldic escutcheon: contour, partition and charge.
    Crest,
    /// Symmetric stained-glass Voronoi tessellation.
    Voronoi,
    /// Kaleidoscopic mandala with N-fold symmetry.
    Kaleido,
    /// Interlaced Lissajous ribbons.
    Ribbon,
    /// Linked stars with a principal star.
    Constellation,
    /// Initials over a generative motif.
    Monogram,
}

impl Engine {
    pub const ALL: [Engine; 7] = [
        Engine::Blob,
        Engine::Crest,
        Engine::Voronoi,
        Engine::Kaleido,
        Engine::Ribbon,
        Engine::Constellation,
        Engine::Monogram,
    ];

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "blob" => Some(Engine::Blob),
            "crest" => Some(Engine::Crest),
            "voronoi" => Some(Engine::Voronoi),
            "kaleido" => Some(Engine::Kaleido),
            "ribbon" => Some(Engine::Ribbon),
            "constellation" => Some(Engine::Constellation),
            "monogram" => Some(Engine::Monogram),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Engine::Blob => "blob",
            Engine::Crest => "crest",
            Engine::Voronoi => "voronoi",
            Engine::Kaleido => "kaleido",
            Engine::Ribbon => "ribbon",
            Engine::Constellation => "constellation",
            Engine::Monogram => "monogram",
        }
    }
}

/// Renders every engine except [`Engine::Blob`], which keeps its historical
/// code path in `render.rs` so existing output stays byte-for-byte stable.
pub fn render(engine: Engine, input: &str, mut ctx: Ctx<'_>, prng: &mut Prng) -> String {
    match engine {
        Engine::Blob => unreachable!("blob engine is rendered by render.rs"),
        Engine::Crest => crest::render(&mut ctx, prng),
        Engine::Voronoi => voronoi::render(&mut ctx, prng),
        Engine::Kaleido => kaleido::render(&mut ctx, prng),
        Engine::Ribbon => ribbon::render(&mut ctx, prng),
        Engine::Constellation => constellation::render(&mut ctx, prng),
        Engine::Monogram => monogram::render(&mut ctx, prng, input),
    }
    ctx.finish();
    ctx.into_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for e in Engine::ALL {
            assert_eq!(Engine::from_name(e.name()), Some(e));
        }
        assert_eq!(Engine::from_name("nope"), None);
    }
}
