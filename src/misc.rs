use bevy::{
    ecs::component::Component,
    math::{IVec2, Vec2},
};
use cosmic_text::{Style, Weight};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "reflect")]
use bevy::{ecs::reflect::ReflectComponent, reflect::Reflect};

/// Horizontal align of text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
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
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct Text3dDimensionOut {
    /// Returns `aabb`'s x and y derived from font's line height.
    pub dimension: Vec2,
    pub(crate) atlas_dimension: IVec2,
}

/// Allows italic or oblique faces to be selected.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub enum TextStyle {
    /// A face that is neither italic not obliqued.
    Normal,
    /// A form that is generally cursive in nature.
    Italic,
    /// A typically-sloped version of the regular face.
    Oblique,
}

impl Default for TextStyle {
    #[inline]
    fn default() -> TextStyle {
        TextStyle::Normal
    }
}

impl Into<Style> for TextStyle {
    fn into(self) -> Style {
        match self {
            TextStyle::Normal => Style::Normal,
            TextStyle::Italic => Style::Italic,
            TextStyle::Oblique => Style::Oblique,
        }
    }
}

impl Into<TextStyle> for Style {
    fn into(self) -> TextStyle {
        match self {
            Style::Normal => TextStyle::Normal,
            Style::Italic => TextStyle::Italic,
            Style::Oblique => TextStyle::Oblique,
        }
    }
}

/// Specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct TextWeight(pub u16);

impl Default for TextWeight {
    #[inline]
    fn default() -> TextWeight {
        TextWeight::NORMAL
    }
}

impl TextWeight {
    /// Thin weight (100), the thinnest value.
    pub const THIN: TextWeight = TextWeight(Weight::THIN.0);
    /// Extra light weight (200).
    pub const EXTRA_LIGHT: TextWeight = TextWeight(Weight::EXTRA_LIGHT.0);
    /// Light weight (300).
    pub const LIGHT: TextWeight = TextWeight(Weight::LIGHT.0);
    /// Normal (400).
    pub const NORMAL: TextWeight = TextWeight(Weight::NORMAL.0);
    /// Medium weight (500, higher than normal).
    pub const MEDIUM: TextWeight = TextWeight(Weight::MEDIUM.0);
    /// Semibold weight (600).
    pub const SEMIBOLD: TextWeight = TextWeight(Weight::SEMIBOLD.0);
    /// Bold weight (700).
    pub const BOLD: TextWeight = TextWeight(Weight::BOLD.0);
    /// Extra-bold weight (800).
    pub const EXTRA_BOLD: TextWeight = TextWeight(Weight::EXTRA_BOLD.0);
    /// Black weight (900), the thickest value.
    pub const BLACK: TextWeight = TextWeight(Weight::BLACK.0);
}

impl Into<Weight> for TextWeight {
    fn into(self) -> Weight {
        Weight(self.0)
    }
}

impl Into<TextWeight> for Weight {
    fn into(self) -> TextWeight {
        TextWeight(self.0)
    }
}
