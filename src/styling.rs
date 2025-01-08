use bevy::{color::Srgba, math::FloatOrd, prelude::Component, sprite::Anchor};
use cosmic_text::{fontdb::ID, Attrs, Family, Style, Weight};
use std::{num::NonZeroU32, sync::Arc};

use crate::{GlyphMeta, TextAlign};

/// Default text style of a rich text component.
#[derive(Debug, Component, Clone)]
pub struct Text3dStyling {
    /// Size of the font, corresponding to world space units.
    ///
    /// Ths is cached per unique value so be sure not to use too many of them.
    pub size: f32,
    /// Name of the font.
    pub font: Arc<str>,
    /// Style of the font.
    pub style: Style,
    /// Weight of the font.
    pub weight: Weight,
    /// Horizontal alignment of the font.
    pub align: TextAlign,
    /// Where local `[0, 0]` is inside the text block's Aabb.
    pub anchor: Anchor,
    /// Color of fill.
    pub color: Srgba,
    /// Color of stroke.
    pub stroke_color: Srgba,
    /// If not set, will not render the fill of the font, making it hollow.
    pub fill: bool,
    /// If set, renders a stroke or outline of the font.
    ///
    /// The value represents a percentage of the font size and should be
    /// in `1..10` for hollow text and `1..20` for outline.
    ///
    /// Ths is cached per unique value so be sure not to use too many of them.
    pub stroke: Option<NonZeroU32>,
    /// The distance between `fill` and `stroke`,
    /// If positive, render stroke in front, if negative, render fill in front.
    /// Only has effect if rendering both fill and stroke.
    ///
    /// The one in front always has transform z `0.0`, the one in the back will have negative z.
    ///
    /// By default this is `-0.5`.
    pub stroke_offset: f32,

    /// Determines what to extract as uv1.
    pub uv1: (GlyphMeta, GlyphMeta),

    /// Tab in terms of spaces, default 4.
    pub tab_width: u16,
}

impl Default for Text3dStyling {
    fn default() -> Self {
        Self {
            size: 16.,
            color: Srgba::WHITE,
            font: Default::default(),
            style: Default::default(),
            weight: Default::default(),
            align: Default::default(),
            anchor: Anchor::Center,
            stroke_color: Srgba::WHITE,
            fill: true,
            stroke: Default::default(),
            stroke_offset: -0.5,
            uv1: (GlyphMeta::Index, GlyphMeta::PerGlyphAdvance),
            tab_width: 4,
        }
    }
}

/// Text style of a segment.
#[derive(Debug, Default, Clone)]
pub struct SegmentStyle {
    pub font: Option<Arc<str>>,
    pub fill_color: Option<Srgba>,
    pub stroke_color: Option<Srgba>,
    pub fill: Option<bool>,
    pub stroke: Option<NonZeroU32>,
    pub bold: bool,
    pub italic: bool,
    /// Can be referenced by [`GlyphMeta::MagicNumber`].
    pub magic_number: Option<f32>,
}

impl SegmentStyle {
    pub fn as_attr(&self) -> Attrs {
        let mut result = Attrs::new();
        if self.bold {
            result = result.weight(Weight::BOLD);
        }
        if self.italic {
            result = result.style(Style::Italic);
        }
        if let Some(name) = self.font.as_ref() {
            result = result.family(Family::Name(name));
        }
        result
    }

    pub fn join(&self, other: Self) -> Self {
        SegmentStyle {
            font: other.font.or_else(|| self.font.clone()),
            fill_color: other.fill_color.or(self.fill_color),
            stroke_color: other.stroke_color.or(self.stroke_color),
            fill: other.fill.or(self.fill),
            stroke: other.stroke.or(self.stroke),
            bold: other.bold | self.bold,
            italic: other.italic | self.italic,
            magic_number: other.magic_number.or(self.magic_number),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphEntry {
    pub font: ID,
    pub glyph_id: u16,
    pub size: FloatOrd,
    pub weight: Weight,
    /// If is none, render in fill mode.
    pub stroke: Option<NonZeroU32>,
}
