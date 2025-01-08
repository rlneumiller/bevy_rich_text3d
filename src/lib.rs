#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
mod change_detection;
mod color_table;
mod fetch;
mod misc;
mod parse;
mod render;
mod styling;
mod text3d;
use bevy::{
    app::{Plugin, PostUpdate},
    asset::{AssetApp, AssetId, Assets},
    image::Image,
    pbr::StandardMaterial,
    prelude::{IntoSystemConfigs, IntoSystemSetConfigs, Resource, SystemSet, TransformSystem},
    text::CosmicFontSystem,
};
use change_detection::TouchMaterialSet;
pub use change_detection::{TouchTextMaterial2dPlugin, TouchTextMaterialPlugin};
pub use fetch::{FetchedTextSegment, TextFetch};
pub use misc::*;
pub use parse::ParseError;
pub use render::{TextAtlas, TextAtlasHandle};
pub use styling::{SegmentStyle, Text3dStyling};
pub use text3d::{Text3d, Text3dSegment};

/// Changes the behavior of [`Text3dPlugin`], should be inserted before it.
#[derive(Debug, Resource)]
pub struct Text3dPluginSettings {
    /// Size of the default font atlas, by default `(512, 512)`, we only extend the atlas by doubling in size vertically.
    ///
    /// Ideally this should be able to contain all glyphs to avoid inefficiencies.
    ///
    /// Trying to cache a glyph bigger than this size will cause a panic.
    pub default_atlas_dimension: (usize, usize),
    /// This should be the primary window's `scale_factor`. For example if this value is 2, a 32 x 32 glyph will
    /// take up 64 x 64 pixels.
    ///
    /// Note the value on `Window` is not real during app creation, so this is up to the user for now.
    pub scale_factor: f32,
}

impl Default for Text3dPluginSettings {
    fn default() -> Self {
        Self {
            default_atlas_dimension: (512, 512),
            scale_factor: 1.0,
        }
    }
}

/// Text3d Plugin, add [`Text3dPluginSettings`] before this to modify its behavior.
pub struct Text3dPlugin;

/// [`SystemSet`] of text3d rendering in [`PostUpdate`] before transforms.
///
/// Manually order this before other transform related systems if applicable.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, SystemSet)]
pub struct Text3dSet;

impl Plugin for Text3dPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TextAtlas>();
        app.init_resource::<Text3dPluginSettings>();
        let (x, y) = app
            .world()
            .resource::<Text3dPluginSettings>()
            .default_atlas_dimension;
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .insert(&TextAtlas::DEFAULT_IMAGE, TextAtlas::empty_image(x, y));
        app.world_mut()
            .resource_mut::<Assets<TextAtlas>>()
            .insert(AssetId::default(), TextAtlas::new(TextAtlas::DEFAULT_IMAGE));
        app.add_systems(
            PostUpdate,
            (fetch::text_fetch_system, render::text_render)
                .chain()
                .in_set(Text3dSet)
                .before(TouchMaterialSet),
        );
        app.configure_sets(
            PostUpdate,
            Text3dSet.before(TransformSystem::TransformPropagate),
        );
        app.configure_sets(PostUpdate, TouchMaterialSet.in_set(Text3dSet));
        app.add_plugins(TouchTextMaterialPlugin::<StandardMaterial>::default());
    }
}

/// Allow [`cosmic_text`] to load system forts.
///
/// Note: this behavior might be supported by bevy directly in the future.
pub struct LoadSystemFontPlugin;

impl Plugin for LoadSystemFontPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.world_mut()
            .resource_mut::<CosmicFontSystem>()
            .0
            .db_mut()
            .load_system_fonts();
    }
}
