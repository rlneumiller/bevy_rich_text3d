#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
mod atlas;
mod change_detection;
mod color_table;
mod fetch;
mod loading;
mod misc;
mod parse;
mod render;
mod styling;
mod text3d;
use std::ops::{Deref, DerefMut};

pub use atlas::{TextAtlas, TextAtlasHandle};
use bevy::{
    app::{First, Plugin, PostUpdate},
    asset::{AssetApp, AssetId, Assets},
    ecs::{
        query::With,
        schedule::common_conditions::resource_exists,
        system::{Query, ResMut},
        world::Ref,
    },
    image::Image,
    prelude::{IntoSystemConfigs, IntoSystemSetConfigs, Resource, SystemSet, TransformSystem},
    window::{PrimaryWindow, Window},
};
use change_detection::TouchMaterialSet;
#[cfg(feature = "2d")]
pub use change_detection::TouchTextMaterial2dPlugin;
#[cfg(feature = "3d")]
pub use change_detection::TouchTextMaterial3dPlugin;
use cosmic_text::FontSystem;
pub use fetch::{FetchedTextSegment, SharedTextSegment, TextFetch};
use loading::{load_cosmic_fonts_system, LoadCosmicFonts};
pub use misc::*;
pub use parse::ParseError;
pub use styling::{SegmentStyle, Text3dStyling};
pub use text3d::{Text3d, Text3dSegment};

fn synchronize_scale_factor(
    mut settings: ResMut<Text3dPlugin>,
    main_window: Query<Ref<Window>, With<PrimaryWindow>>,
    mut atlases: ResMut<Assets<TextAtlas>>,
    mut images: ResMut<Assets<Image>>,
) {
    if settings.sync_scale_factor_with_main_window {
        if let Ok(window) = main_window.get_single() {
            if window.scale_factor() != settings.scale_factor {
                settings.scale_factor = window.scale_factor();
                for (_, atlas) in atlases.iter_mut() {
                    atlas.clear(&mut images);
                }
            }
        }
    }
}

/// Text3d Plugin, add [`Text3dPluginSettings`] before this to modify its behavior.
#[derive(Debug, Resource, Clone)]
pub struct Text3dPlugin {
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
    /// Currently the [`Window`]'s scale factor is not correct at app startup,
    /// if true this synchronizes scale factor with the [`PrimaryWindow`]'s stats.
    pub sync_scale_factor_with_main_window: bool,
    /// System locale, like `en-US`.
    pub locale: Option<String>,
    /// If true, load system fonts,
    pub load_system_fonts: bool,
    /// Path of fonts to be loaded.
    pub load_font_paths: Vec<String>,
    /// Path of font directories to be loaded.
    pub load_font_directories: Vec<String>,
    /// Fonts embedded in the executable.
    pub load_font_embedded: Vec<&'static [u8]>,
    /// If false, may increase the app's startup time,
    ///
    /// If true,
    /// load fonts concurrently on `IOTaskPool` and
    ///  [`Text3dCosmicFontSystem`] might not be available immediately.
    pub asynchronous_load: bool,
}

impl Default for Text3dPlugin {
    fn default() -> Self {
        Self {
            default_atlas_dimension: (512, 512),
            scale_factor: 1.0,
            sync_scale_factor_with_main_window: true,
            load_system_fonts: false,
            load_font_paths: vec![],
            load_font_directories: vec![],
            load_font_embedded: vec![],
            asynchronous_load: false,
            locale: None,
        }
    }
}

/// [`SystemSet`] of text3d rendering in [`PostUpdate`] before transforms.
///
/// Manually order this before other transform related systems if applicable.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, SystemSet)]
pub struct Text3dSet;

#[derive(Debug, Resource)]
pub struct Rt3dCosmicFontSystem(FontSystem);

impl Deref for Rt3dCosmicFontSystem {
    type Target = FontSystem;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Rt3dCosmicFontSystem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Plugin for Text3dPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TextAtlas>();
        app.insert_resource::<Text3dPlugin>(self.clone());
        if self.asynchronous_load {
            app.insert_resource(self.load_fonts_concurrent());
        } else {
            app.insert_resource(self.load_fonts_blocking());
        }
        let (x, y) = self.default_atlas_dimension;
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .insert(&TextAtlas::DEFAULT_IMAGE, TextAtlas::empty_image(x, y));
        app.world_mut()
            .resource_mut::<Assets<TextAtlas>>()
            .insert(AssetId::default(), TextAtlas::new(TextAtlas::DEFAULT_IMAGE));
        app.add_systems(First, synchronize_scale_factor);
        app.add_systems(
            First,
            load_cosmic_fonts_system.run_if(resource_exists::<LoadCosmicFonts>),
        );
        app.add_systems(
            PostUpdate,
            (
                fetch::text_fetch_system,
                render::text_render.run_if(resource_exists::<Rt3dCosmicFontSystem>),
            )
                .chain()
                .in_set(Text3dSet)
                .before(TouchMaterialSet),
        );
        app.configure_sets(
            PostUpdate,
            Text3dSet.before(TransformSystem::TransformPropagate),
        );
        app.configure_sets(PostUpdate, TouchMaterialSet.in_set(Text3dSet));
        #[cfg(feature = "2d")]
        app.add_plugins(TouchTextMaterial2dPlugin::<bevy::sprite::ColorMaterial>::default());
        #[cfg(feature = "3d")]
        app.add_plugins(TouchTextMaterial3dPlugin::<bevy::pbr::StandardMaterial>::default());
    }
}
