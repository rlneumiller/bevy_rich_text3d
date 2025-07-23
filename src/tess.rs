use bevy::{
    image::Image,
    math::{IVec2, Rect, Vec2},
};
use cosmic_text::ttf_parser::OutlineBuilder;
use zeno::{Cap, Command, Format, Mask, Stroke, Style, Transform, Vector};

use crate::{styling::GlyphEntry, TextAtlas};

#[derive(Debug, Default)]
pub(crate) struct CommandEncoder {
    pub commands: Vec<Command>,
}

impl OutlineBuilder for CommandEncoder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(Command::MoveTo(Vector::new(x, y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(Command::LineTo(Vector::new(x, y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.commands
            .push(Command::QuadTo(Vector::new(x1, y1), Vector::new(x, y)));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.commands.push(Command::CurveTo(
            Vector::new(x1, y1),
            Vector::new(x2, y2),
            Vector::new(x, y),
        ));
    }

    fn close(&mut self) {
        self.commands.push(Command::Close);
    }
}

impl CommandEncoder {
    /// Returns a rectangle and an additional offset, keep in mind both has to be applied scale factor before usage.
    pub fn tess_glyph(
        &self,
        stroke: Option<f32>,
        scale: f32,
        atlas: &mut TextAtlas,
        image: &mut Image,
        entry: GlyphEntry,
    ) -> Option<(Rect, Vec2)> {
        let (alpha_map, bb) = if let Some(stroke) = stroke {
            Mask::new(&self.commands)
                .style(Style::Stroke(Stroke {
                    width: stroke,
                    start_cap: Cap::Round,
                    end_cap: Cap::Round,
                    join: entry.join.into(),
                    ..Default::default()
                }))
                .transform(Some(Transform::scale(scale, scale)))
                .format(Format::Alpha)
                .render()
        } else {
            Mask::new(&self.commands)
                .transform(Some(Transform::scale(scale, scale)))
                .format(Format::Alpha)
                .render()
        };
        let (w, h) = (bb.width as usize, bb.height as usize);
        let base = Vec2::new(bb.left as f32, bb.top as f32);
        let pixel_rect = atlas.cache(image, entry, base, w, h, |buffer, pitch| {
            for x in 0..w {
                for y in 0..h {
                    buffer[y * pitch + x * 4 + 3] = alpha_map[y * w + x]
                }
            }
            IVec2::new(w as i32, h as i32)
        });
        Some((pixel_rect, base))
    }
}
