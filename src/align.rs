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
    /// Index of the glyph, `0`, `1`, etc.
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

#[derive(Debug, Component)]
pub struct Text3dBounds {
    pub width: f32,
}

impl Default for Text3dBounds {
    fn default() -> Self {
        Self { width: f32::MAX }
    }
}

#[derive(Debug, Component, Default)]
pub struct Text3dDimensionOut {
    pub dimension: Vec2,
    pub(crate) atlas_dimension: IVec2,
}
