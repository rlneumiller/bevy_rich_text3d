use std::ops::{Deref, DerefMut};

use bevy::{
    math::{IVec2, Vec2},
    prelude::Component,
};

/// Horizontal align of text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl TextAlign {
    pub fn as_fac(&self) -> f32 {
        match self {
            TextAlign::Left => 0.,
            TextAlign::Center => 0.5,
            TextAlign::Right => 1.0,
        }
    }
}

/// Determines what kind of data each field in `uv1` carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlyphMeta {
    /// Left to right count of the glyph, `0`, `1`, etc.
    #[default]
    Index,
    /// Returns x position in `em` of a vertex as if the text is rendered in a single line.
    Advance,
    /// Returns x position in `em` of the center of a glyph as if the text is rendered in a single line.
    PerGlyphAdvance,
    /// The `uv.x` as if the text block is a rectangular sprite.
    RowX,
    /// The `uv.y` as if the text block is a rectangular sprite.
    ColY,
    /// The [`SegmentStyle::magic_number`](crate::SegmentStyle::magic_number) field
    MagicNumber,
}

/// Determines the maximum width of rendered text, by default infinite.
#[derive(Debug, Component)]
pub struct Text3dBounds {
    pub width: f32,
}

impl Default for Text3dBounds {
    fn default() -> Self {
        Self { width: f32::MAX }
    }
}

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

/// Size of the output mesh's `Aabb`.
#[derive(Debug, Component, Default)]
pub struct Text3dDimensionOut {
    pub dimension: Vec2,
    pub(crate) atlas_dimension: IVec2,
}
