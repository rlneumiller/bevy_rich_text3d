use bevy::{
    color::Srgba,
    math::{FloatOrd, Vec2},
    prelude::Component,
};
use cosmic_text::{fontdb::ID, Attrs, Family, Style, Weight};
use std::{
    num::NonZeroU32,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{GlyphMeta, TextAlign};

/// Anchor of a text block, usually in `(-0.5, -0.5)..=(0.5, 0.5)`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextAnchor(pub Vec2);

impl Deref for TextAnchor {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextAnchor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TextAnchor {
    pub const BOTTOM_LEFT: TextAnchor = TextAnchor::new(-0.5, -0.5);
    pub const BOTTOM_CENTER: TextAnchor = TextAnchor::new(0., -0.5);
    pub const BOTTOM_RIGHT: TextAnchor = TextAnchor::new(0.5, -0.5);

    pub const CENTER_LEFT: TextAnchor = TextAnchor::new(-0.5, 0.);
    pub const CENTER: TextAnchor = TextAnchor::new(0., 0.);
    pub const CENTER_RIGHT: TextAnchor = TextAnchor::new(0.5, 0.);

    pub const TOP_LEFT: TextAnchor = TextAnchor::new(-0.5, 0.5);
    pub const TOP_CENTER: TextAnchor = TextAnchor::new(0., 0.5);
    pub const TOP_RIGHT: TextAnchor = TextAnchor::new(0.5, 0.5);

    pub const fn new(x: f32, y: f32) -> Self {
        TextAnchor(Vec2::new(x, y))
    }
}

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
    pub anchor: TextAnchor,
    /// Height of a line multiplied to font size, by default `1.0`.
    pub line_height: f32,
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
            anchor: TextAnchor::CENTER,
            stroke_color: Srgba::WHITE,
            fill: true,
            stroke: Default::default(),
            line_height: 1.0,
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
    pub weight: Option<Weight>,
    pub style: Option<Style>,
    /// Can be referenced by [`GlyphMeta::MagicNumber`].
    pub magic_number: Option<f32>,
}

impl SegmentStyle {
    pub fn as_attr<'t>(&'t self, base: &'t Text3dStyling) -> Attrs<'t> {
        Attrs::new()
            .weight(self.weight.unwrap_or(base.weight))
            .style(self.style.unwrap_or(base.style))
            .family(Family::Name(
                self.font.as_ref().map(Arc::as_ref).unwrap_or(&base.font),
            ))
    }

    pub fn join(&self, other: Self) -> Self {
        SegmentStyle {
            font: other.font.or_else(|| self.font.clone()),
            fill_color: other.fill_color.or(self.fill_color),
            stroke_color: other.stroke_color.or(self.stroke_color),
            fill: other.fill.or(self.fill),
            stroke: other.stroke.or(self.stroke),
            weight: other.weight.or(self.weight),
            style: other.style.or(self.style),
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
