use identicon_core::{RenderError, RenderOptions, Theme};
use wasm_bindgen::prelude::*;

fn map_error(err: RenderError) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[wasm_bindgen]
pub fn render_identicon(input: &str) -> Result<String, JsValue> {
    identicon_core::try_render_identicon(input).map_err(map_error)
}

#[wasm_bindgen]
pub fn render_identicon_with_options(
    input: &str,
    size: u32,
    theme: Option<String>,
    background: bool,
    animated: bool,
) -> Result<String, JsValue> {
    let theme = match theme {
        Some(name) => {
            Some(Theme::from_name(&name).ok_or_else(|| JsValue::from_str("unknown theme"))?)
        }
        None => None,
    };

    let opts = RenderOptions {
        size,
        theme,
        background,
        animated,
    };

    identicon_core::render_identicon_with_options(input, &opts).map_err(map_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_exports_match_core() {
        let svg = render_identicon("alice").unwrap();
        assert!(svg.starts_with("<svg"));
    }
}
