use bevy::{asset::{Asset, AssetId, Assets, Handle, RenderAssetUsages}, ecs::component::Component, image::Image, math::{IVec2, Rect, Vec2}, reflect::TypePath, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, utils::HashMap};

use crate::styling::GlyphEntry;


/// Backing image handle and atlas of [`Text3d`].
#[derive(Debug, Clone, Default, TypePath, Asset)]
pub struct TextAtlas {
    pub(crate) image: Handle<Image>,
    pub(crate) glyphs: HashMap<GlyphEntry, (Rect, Vec2)>,
    pub(crate) pointer: IVec2,
    pub(crate) descent: usize,
}

const PADDING: usize = 2;

impl TextAtlas {
    /// The image used by [`TextAtlas::default()`].
    pub const DEFAULT_IMAGE: Handle<Image> =
        Handle::weak_from_u128(0x9a5c50eb057602509c7836bb327807e1);

    /// Create a new empty [`TextAtlas`].
    ///
    /// The image can be created via [`TextAtlas::empty_image`].
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            image,
            ..Default::default()
        }
    }

    /// Create an empty [`Image`] filled with transparent white `(255, 255, 255, 0)`.
    pub fn empty_image(width: usize, height: usize) -> Image {
        Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![[255, 255, 255, 0]; width * height].into_flattened(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::all(),
        )
    }

    /// Cache a glyph.
    pub fn cache(
        &mut self,
        images: &mut Assets<Image>,
        id: AssetId<Image>,
        glyph: GlyphEntry,
        base: Vec2,
        width: usize,
        height: usize,
        mut draw: impl FnMut(&mut [u8], usize) -> IVec2,
    ) -> Rect {
        if let Some((rect, _)) = self.glyphs.get(&glyph) {
            return *rect;
        }
        let Some(image) = images.get_mut(id) else {
            return Rect::default();
        };
        if self.pointer.x as usize + width + PADDING > image.width() as usize {
            self.pointer.x = 0;
            self.pointer.y += self.descent.max(height) as i32 + PADDING as i32;
            self.descent = 0;
        }
        self.descent = self.descent.max(height);
        if self.pointer.y as usize + self.descent + PADDING >= image.height() as usize {
            let old_dim = (image.width() * image.height()) as usize;
            image.resize(Extent3d {
                width: image.width(),
                height: image.height() * 2,
                depth_or_array_layers: 1,
            });
            for i in old_dim..old_dim * 2 {
                image.data[i * 4] = 255;
                image.data[i * 4 + 1] = 255;
                image.data[i * 4 + 2] = 255;
            }
        };
        let w = image.width() as usize;

        let dimension = draw(
            &mut image.data[(self.pointer.y as usize * w + self.pointer.x as usize) * 4..],
            w * 4,
        );

        let output = Rect {
            min: self.pointer.as_vec2(),
            max: (self.pointer + dimension).as_vec2(),
        };

        self.glyphs.insert(glyph, (output, base));
        self.pointer.x += dimension.x + PADDING as i32;

        output
    }

    /// Clear all cached glyphs and repaint the image as transparent white.
    pub fn clear(&mut self, images: &mut Assets<Image>) {
        self.pointer = IVec2::ZERO;
        self.glyphs.clear();
        if let Some(img) = images.get_mut(self.image.id()) {
            for chunk in img.data.chunks_mut(4) {
                chunk[0] = 255;
                chunk[1] = 255;
                chunk[2] = 255;
                chunk[3] = 0;
            }
        }
    }
}

/// [`Component`] of a [`Handle<TextAtlas>`](TextAtlas), if left as default,
/// will use the shared [`TextAtlas::DEFAULT_IMAGE`] as
/// the underlying image.
#[derive(Debug, Clone, Component, Default)]
pub struct TextAtlasHandle(pub Handle<TextAtlas>);