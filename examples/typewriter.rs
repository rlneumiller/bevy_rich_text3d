use bevy::{
    app::{App, Startup},
    asset::{Asset, Assets},
    color::Color,
    math::Vec3,
    pbr::{
        AmbientLight, ExtendedMaterial, MaterialExtension, MaterialPlugin, MeshMaterial3d,
        StandardMaterial,
    },
    prelude::{
        AlphaMode, Camera3d, Commands, Mesh3d, OrthographicProjection, Projection, Res, ResMut,
        Transform,
    },
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    time::{Time, Virtual},
    DefaultPlugins,
};
use bevy_rich_text3d::{
    GlyphMeta, LoadSystemFontPlugin, Text3d, Text3dBounds, Text3dPlugin, Text3dPluginSettings,
    Text3dStyling, TextAlign, DEFAULT_GLYPH_ATLAS,
};

#[derive(Debug, Clone, TypePath, AsBindGroup, Asset)]
pub struct TypewriterShader {
    #[uniform(100)]
    pub from: f32,
    #[uniform(101)]
    pub speed: f32,
}

impl MaterialExtension for TypewriterShader {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path("typewriter.wgsl".into())
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, TypewriterShader>>::default())
        .insert_resource(Text3dPluginSettings {
            default_atlas_dimension: (1024, 512),
            scale_factor: 2.,
        })
        .add_plugins(Text3dPlugin)
        .add_plugins(LoadSystemFontPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 800.,
        })
        .add_systems(Startup, |
            mut commands: Commands,
            time: Res<Time<Virtual>>,
            mut mats: ResMut<Assets<ExtendedMaterial<StandardMaterial, TypewriterShader>>>,
        | {
            let mat = mats.add(
                ExtendedMaterial {
                    base: StandardMaterial {
                        base_color_texture: Some(DEFAULT_GLYPH_ATLAS.clone_weak()),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..Default::default()
                    },
                    extension: TypewriterShader {
                        from: time.elapsed_secs(),
                        speed: 4.0,
                    },
                }
            );
            commands.spawn((
                Text3d::new(include_str!("lorem.txt")),
                Text3dStyling {
                    align: TextAlign::Left,
                    uv1: (GlyphMeta::PerGlyphAdvance, GlyphMeta::Advance),
                    ..Default::default()
                },
                Text3dBounds { width: 500. },
                Mesh3d::default(),
                MeshMaterial3d(mat.clone()),
            ));
            commands.spawn((
                Camera3d::default(),
                Projection::Orthographic(OrthographicProjection::default_3d()),
                Transform::from_translation(Vec3::new(0., 0., 1.))
                    .looking_at(Vec3::new(0., 0., 0.), Vec3::Y)
            ));
        })
        .run();
}
