mod bezier;
mod blob;
mod hash;
mod palette;
mod prng;
mod render;
mod svg;

pub use palette::{PALETTE_COUNT, PALETTES, Theme, palette_by_index, palette_by_theme};
pub use render::render_identicon_inner;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    EmptyInput,
    InputTooLong,
    InvalidCharset,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::EmptyInput => write!(f, "input must not be empty"),
            RenderError::InputTooLong => write!(f, "input exceeds maximum length"),
            RenderError::InvalidCharset => write!(f, "input contains invalid characters"),
        }
    }
}

impl std::error::Error for RenderError {}

#[derive(Clone, Debug)]
pub struct RenderOptions {
    pub size: u32,
    pub theme: Option<Theme>,
    pub background: bool,
    pub animated: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            size: 256,
            theme: None,
            background: true,
            animated: false,
        }
    }
}

/// Renders an identicon for valid input. Invalid input returns an empty SVG placeholder.
pub fn render_identicon(input: &str) -> String {
    render_identicon_with_options(input, &RenderOptions::default()).unwrap_or_else(|_| {
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256"></svg>"#.into()
    })
}

pub fn render_identicon_with_options(
    input: &str,
    opts: &RenderOptions,
) -> Result<String, RenderError> {
    render::render_identicon_inner(input, opts)
}

pub fn try_render_identicon(input: &str) -> Result<String, RenderError> {
    render_identicon_with_options(input, &RenderOptions::default())
}
