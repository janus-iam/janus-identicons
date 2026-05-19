use crate::bezier::write_f32;
use crate::blob::{Accent, BackgroundStyle, RenderedBlob, palette_color};
use crate::palette::Palette;
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

    pub fn write_background_gradient(
        &mut self,
        palette: &Palette,
        style: BackgroundStyle,
        gradient_angle: f32,
    ) {
        let (bg_start, bg_end) = match style {
            BackgroundStyle::Light => (palette.bg_light, palette.colors[0]),
            BackgroundStyle::Dark => (palette.bg_dark, palette.colors[2]),
        };

        let rad = gradient_angle.to_radians();
        let x2 = rad.cos();
        let y2 = rad.sin();

        self.out
            .push_str("<linearGradient id=\"bg\" x1=\"0\" y1=\"0\" x2=\"");
        write_f32(&mut self.out, x2);
        self.out.push_str("\" y2=\"");
        write_f32(&mut self.out, y2);
        self.out.push_str("\">");
        write_simple_stop(&mut self.out, 0.0, bg_start, 1.0);
        write_simple_stop(&mut self.out, 1.0, bg_end, 0.35);
        self.out.push_str("</linearGradient>");
    }

    pub fn write_background_rect(&mut self) {
        self.out
            .push_str(r#"<rect width="100%" height="100%" fill="url(#bg)"/>"#);
    }

    pub fn write_blob_gradient(
        &mut self,
        id: &str,
        blob: &RenderedBlob,
        palette: &Palette,
        size: u32,
    ) {
        let c0 = palette_color(palette, blob.color_indices[0]);
        let c1 = palette_color(palette, blob.color_indices[1]);
        let c2 = palette_color(palette, blob.color_indices[2]);

        if blob.use_radial {
            self.out.push_str("<radialGradient id=\"");
            self.out.push_str(id);
            self.out.push_str("\" cx=\"");
            write_f32(&mut self.out, blob.cx / size as f32);
            self.out.push_str("\" cy=\"");
            write_f32(&mut self.out, blob.cy / size as f32);
            self.out.push_str("\" r=\"0.5\">");
            write_simple_stop(&mut self.out, 0.0, c0, blob.opacity);
            write_simple_stop(&mut self.out, 0.55, c1, blob.opacity * 0.75);
            write_simple_stop(&mut self.out, 1.0, c2, 0.15);
            self.out.push_str("</radialGradient>");
        } else {
            self.out.push_str("<linearGradient id=\"");
            self.out.push_str(id);
            self.out.push_str("\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\">");
            write_simple_stop(&mut self.out, 0.0, c0, blob.opacity);
            write_simple_stop(&mut self.out, 0.5, c1, blob.opacity * 0.8);
            write_simple_stop(&mut self.out, 1.0, c2, 0.2);
            if self.animated {
                let dur = 6 + (self.anim_seed % 5);
                self.out
                    .push_str("<animate attributeName=\"x1\" values=\"0;0.3;0\" dur=\"");
                write_u32(&mut self.out, dur);
                self.out.push_str("s\" repeatCount=\"indefinite\"/>");
            }
            self.out.push_str("</linearGradient>");
        }
    }

    pub fn write_blob_path(&mut self, grad_id: &str, blob: &RenderedBlob) {
        self.out.push_str("<path fill=\"url(#");
        self.out.push_str(grad_id);
        self.out.push_str(")\" d=\"");
        self.out.push_str(&blob.path_d);
        self.out.push('"');
        if self.animated {
            self.out.push_str(
                " opacity=\"0.85\"><animate attributeName=\"opacity\" values=\"0.75;0.95;0.75\" dur=\"5s\" repeatCount=\"indefinite\"/></path>",
            );
        } else {
            self.out.push_str("/>");
        }
    }

    pub fn write_accent(&mut self, palette: &Palette, accent: &Accent, idx: usize) {
        let color = palette_color(palette, accent.color_index);
        self.out.push_str("<circle cx=\"");
        write_f32(&mut self.out, accent.x);
        self.out.push_str("\" cy=\"");
        write_f32(&mut self.out, accent.y);
        self.out.push_str("\" r=\"");
        write_f32(&mut self.out, accent.r);
        self.out.push_str("\" fill=\"");
        self.out.push_str(color);
        self.out.push_str("\" fill-opacity=\"");
        write_f32(&mut self.out, accent.opacity);
        self.out.push('"');
        if self.animated && idx == 0 {
            self.out.push_str("><animate attributeName=\"r\" values=\"");
            write_f32(&mut self.out, accent.r);
            self.out.push(';');
            write_f32(&mut self.out, accent.r * 1.2);
            self.out.push(';');
            write_f32(&mut self.out, accent.r);
            self.out
                .push_str("\" dur=\"4s\" repeatCount=\"indefinite\"/></circle>");
        } else {
            self.out.push_str("/>");
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
