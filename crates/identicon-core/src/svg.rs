use crate::bezier::write_f32;
use crate::palette::Palette;
use crate::sigil::{palette_color, ArcLayer, BackgroundStyle, CenterGlyph, OrbitNode};
use crate::sigil::{write_arc_path, write_center_glyph};

pub struct SvgBuilder {
    out: String,
    animated: bool,
    anim_seed: u32,
}

impl SvgBuilder {
    pub fn new(capacity: usize, animated: bool, anim_seed: u32) -> Self {
        Self {
            out: String::with_capacity(capacity),
            animated,
            anim_seed,
        }
    }

    pub fn into_string(self) -> String {
        self.out
    }

    pub fn begin(&mut self, size: u32) {
        self.out
            .push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 "#);
        write_u32(&mut self.out, size);
        self.out.push(' ');
        write_u32(&mut self.out, size);
        self.out.push_str(r#"" width=""#);
        write_u32(&mut self.out, size);
        self.out.push_str(r#"" height=""#);
        write_u32(&mut self.out, size);
        self.out
            .push_str(r#"" role="img" aria-hidden="true"><defs>"#);
    }

    pub fn close_defs(&mut self) {
        self.out.push_str("</defs>");
    }

    pub fn finish(&mut self) {
        self.out.push_str("</svg>");
    }

    pub fn write_background(
        &mut self,
        palette: &Palette,
        style: BackgroundStyle,
        size: u32,
    ) {
        let (bg, accent) = match style {
            BackgroundStyle::Light => (palette.bg_light, palette.colors[0]),
            BackgroundStyle::Dark => (palette.bg_dark, palette.colors[1]),
        };

        self.out.push_str("<radialGradient id=\"bg\" cx=\"0.5\" cy=\"0.5\" r=\"0.72\">");
        write_simple_stop(&mut self.out, 0.0, bg, 1.0);
        write_simple_stop(&mut self.out, 1.0, accent, 0.2);
        self.out.push_str("</radialGradient>");

        let _ = size;
    }

    pub fn write_background_rect(&mut self) {
        self.out
            .push_str(r#"<rect width="100%" height="100%" fill="url(#bg)"/>"#);
    }

    pub fn write_guide_rings(&mut self, cx: f32, cy: f32, size: f32, color: &str, opacity: f32) {
        let max_r = size * 0.44;
        for i in 1..=3 {
            let r = max_r * (i as f32 / 3.0);
            self.out.push_str("<circle cx=\"");
            write_f32(&mut self.out, cx);
            self.out.push_str("\" cy=\"");
            write_f32(&mut self.out, cy);
            self.out.push_str("\" r=\"");
            write_f32(&mut self.out, r);
            self.out.push_str("\" fill=\"none\" stroke=\"");
            self.out.push_str(color);
            self.out.push_str("\" stroke-opacity=\"");
            write_f32(&mut self.out, opacity);
            self.out.push_str("\" stroke-width=\"0.6\"/>");
        }
    }

    pub fn begin_orbit_group(&mut self, cx: f32, cy: f32, layer: u32) {
        self.out.push_str("<g");
        if self.animated {
            let dur = 18 + ((self.anim_seed + layer) % 12);
            let dir = if layer.is_multiple_of(2) { 1.0 } else { -1.0 };
            self.out.push_str("><animateTransform attributeName=\"transform\" type=\"rotate\" from=\"0 ");
            write_f32(&mut self.out, cx);
            self.out.push(' ');
            write_f32(&mut self.out, cy);
            self.out.push_str("\" to=\"");
            write_f32(&mut self.out, 360.0 * dir);
            self.out.push(' ');
            write_f32(&mut self.out, cx);
            self.out.push(' ');
            write_f32(&mut self.out, cy);
            self.out.push_str("\" dur=\"");
            write_u32(&mut self.out, dur);
            self.out.push_str("s\" repeatCount=\"indefinite\"/>");
        }
        self.out.push('>');
    }

    pub fn end_group(&mut self) {
        self.out.push_str("</g>");
    }

    pub fn write_arc(
        &mut self,
        cx: f32,
        cy: f32,
        layer: &ArcLayer,
        palette: &Palette,
    ) {
        let color = palette_color(palette, layer.color_index);
        self.out.push_str("<path d=\"");
        write_arc_path(&mut self.out, cx, cy, layer);
        self.out.push_str("\" fill=\"none\" stroke=\"");
        self.out.push_str(color);
        self.out.push_str("\" stroke-width=\"");
        write_f32(&mut self.out, layer.stroke_width);
        self.out.push_str("\" stroke-opacity=\"");
        write_f32(&mut self.out, layer.opacity);
        self.out.push_str("\" stroke-linecap=\"round\"");
        if self.animated {
            let dur = 5 + (self.anim_seed % 4);
            self.out.push_str("><animate attributeName=\"stroke-opacity\" values=\"");
            write_f32(&mut self.out, layer.opacity * 0.7);
            self.out.push(';');
            write_f32(&mut self.out, layer.opacity);
            self.out.push(';');
            write_f32(&mut self.out, layer.opacity * 0.7);
            self.out.push_str("\" dur=\"");
            write_u32(&mut self.out, dur);
            self.out.push_str("s\" repeatCount=\"indefinite\"/></path>");
        } else {
            self.out.push_str("/>");
        }
    }

    pub fn write_orbit_node(&mut self, cx: f32, cy: f32, node: &OrbitNode, palette: &Palette) {
        let (x, y) = (
            cx + node.radius * node.angle.cos(),
            cy + node.radius * node.angle.sin(),
        );
        let color = palette_color(palette, node.color_index);
        self.out.push_str("<circle cx=\"");
        write_f32(&mut self.out, x);
        self.out.push_str("\" cy=\"");
        write_f32(&mut self.out, y);
        self.out.push_str("\" r=\"");
        write_f32(&mut self.out, node.radius_px);
        self.out.push_str("\" fill=\"");
        self.out.push_str(color);
        self.out.push_str("\" fill-opacity=\"");
        write_f32(&mut self.out, node.opacity);
        self.out.push('"');
        if self.animated {
            self.out.push_str("><animate attributeName=\"r\" values=\"");
            write_f32(&mut self.out, node.radius_px * 0.85);
            self.out.push(';');
            write_f32(&mut self.out, node.radius_px * 1.15);
            self.out.push(';');
            write_f32(&mut self.out, node.radius_px * 0.85);
            self.out.push_str("\" dur=\"3s\" repeatCount=\"indefinite\"/></circle>");
        } else {
            self.out.push_str("/>");
        }
    }

    pub fn write_center(
        &mut self,
        glyph: CenterGlyph,
        cx: f32,
        cy: f32,
        size: f32,
        palette: &Palette,
        color_index: usize,
    ) {
        let color = palette_color(palette, color_index);
        if self.animated {
            self.out.push_str("<g><animateTransform attributeName=\"transform\" type=\"rotate\" from=\"0 ");
            write_f32(&mut self.out, cx);
            self.out.push(' ');
            write_f32(&mut self.out, cy);
            self.out.push_str("\" to=\"360 ");
            write_f32(&mut self.out, cx);
            self.out.push(' ');
            write_f32(&mut self.out, cy);
            self.out.push_str("\" dur=\"24s\" repeatCount=\"indefinite\"/>");
            write_center_glyph(&mut self.out, glyph, cx, cy, size, color, 0.95);
            self.out.push_str("</g>");
        } else {
            write_center_glyph(&mut self.out, glyph, cx, cy, size, color, 0.95);
        }
    }
}

fn write_simple_stop(out: &mut String, offset: f32, color: &str, opacity: f32) {
    out.push_str("<stop offset=\"");
    write_f32(out, offset);
    out.push_str("\" stop-color=\"");
    out.push_str(color);
    out.push_str("\" stop-opacity=\"");
    write_f32(out, opacity);
    out.push_str("\"/>");
}

fn write_u32(out: &mut String, v: u32) {
    out.push_str(&v.to_string());
}

pub fn clamp_size(size: u32) -> u32 {
    size.clamp(32, 1024)
}
