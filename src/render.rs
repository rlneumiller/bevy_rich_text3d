use bevy::{
    asset::{Asset, AssetId, Assets, Handle, RenderAssetUsages},
    color::{ColorToComponents, LinearRgba},
    image::Image,
    math::{FloatOrd, IVec2, Rect, Vec2},
    prelude::{Component, DetectChanges, Mesh, Mesh2d, Mesh3d, Mut, Query, Ref, Res, ResMut},
    reflect::TypePath,
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    text::CosmicFontSystem,
    utils::HashMap,
};
use cosmic_text::{
    ttf_parser::{Face, GlyphId, OutlineBuilder},
    Attrs, Buffer, Family, Metrics, Shaping, Wrap,
};
use zeno::{Cap, Command, Format, Mask, Stroke, Style, Transform, Vector};

use crate::{
    fetch::FetchedTextSegment,
    styling::GlyphEntry,
    text3d::{Text3d, Text3dSegment},
    GlyphMeta, Text3dBounds, Text3dDimensionOut, Text3dPluginSettings, Text3dStyling,
};

fn corners(rect: Rect) -> [[f32; 2]; 4] {
    [
        [rect.min.x, rect.min.y],
        [rect.max.x, rect.min.y],
        [rect.min.x, rect.max.y],
        [rect.max.x, rect.max.y],
    ]
}

#[derive(Debug, Clone, Default, TypePath, Asset)]
pub struct TextAtlas {
    pub image: Handle<Image>,
    pub glyphs: HashMap<GlyphEntry, Rect>,
    pointer: IVec2,
    descent: usize,
}

pub(crate) fn new_image(width: usize, height: usize) -> Image {
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

const PADDING: usize = 2;

impl TextAtlas {
    pub fn from_image(image: Handle<Image>) -> Self {
        Self {
            image,
            ..Default::default()
        }
    }

    pub fn cache(
        &mut self,
        images: &mut Assets<Image>,
        id: AssetId<Image>,
        glyph: GlyphEntry,
        width: usize,
        height: usize,
        mut draw: impl FnMut(&mut [u8], usize) -> IVec2,
    ) -> Rect {
        if let Some(rect) = self.glyphs.get(&glyph) {
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

        self.glyphs.insert(glyph, output);
        self.pointer.x += dimension.x + PADDING as i32;

        output
    }
}

#[derive(Debug, Clone, Component, Default)]
pub struct TextAtlasHandle(pub Handle<TextAtlas>);

// Take the allocation if possible but clear the data.
macro_rules! recycle_mesh {
    ($mesh: expr, $attr: ident, $ty: ident) => {
        if let Some(VertexAttributeValues::$ty(mut v)) = $mesh.remove_attribute(Mesh::$attr) {
            v.clear();
            v
        } else {
            Vec::new()
        }
    };
}

fn get_mesh<'t>(
    mesh2d: &mut Option<Mut<Mesh2d>>,
    mesh3d: &mut Option<Mut<Mesh3d>>,
    meshes: &'t mut Assets<Mesh>,
) -> Option<&'t mut Mesh> {
    let mut id = mesh2d
        .as_ref()
        .map(|x| x.id())
        .or_else(|| mesh3d.as_ref().map(|x| x.id()))?;
    if id == AssetId::default() {
        let handle = meshes.add(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        ));
        id = handle.id();
        if let Some(handle_2d) = mesh2d {
            handle_2d.0 = handle.clone();
        }
        if let Some(handle_3d) = mesh3d {
            handle_3d.0 = handle;
        }
    }
    meshes.get_mut(id)
}

/// Returns dimension, offset and min
fn center_aabb_on_anchor(items: &[[f32; 3]], anchor: Vec2) -> (Vec2, Vec2, Vec2) {
    let mut min = Vec2::MAX;
    let mut max = Vec2::MIN;
    for chunk in items.chunks(4) {
        min = min.min(Vec2::new(chunk[0][0], chunk[0][1]));
        max = max.max(Vec2::new(chunk[3][0], chunk[3][1]));
    }
    (max - min, anchor * (max - min) - (max + min) / 2., min)
}

pub fn text_render(
    settings: Res<Text3dPluginSettings>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextAtlas>>,
    mut text_query: Query<(
        Ref<Text3d>,
        Ref<Text3dBounds>,
        Ref<Text3dStyling>,
        &TextAtlasHandle,
        Option<&mut Mesh2d>,
        Option<&mut Mesh3d>,
        &mut Text3dDimensionOut,
    )>,
    segments: Query<Ref<FetchedTextSegment>>,
) {
    let scale_factor = settings.scale_factor;
    for (text, bounds, styling, atlas, mut mesh2d, mut mesh3d, mut output) in text_query.iter_mut()
    {
        let Some(atlas) = atlases.get_mut(atlas.0.id()) else {
            return;
        };

        if atlas.image.id() == AssetId::default() || !images.contains(atlas.image.id()) {
            atlas.image = images.add(new_image(
                settings.default_atlas_dimension.0,
                settings.default_atlas_dimension.1,
            ))
        };

        // Change detection.
        if !text.is_changed() && !bounds.is_changed() && !styling.is_changed() {
            let mut unchanged = true;
            for segment in &text.segments {
                if let Text3dSegment::Extract(entity) = &segment.0 {
                    if segments.get(*entity).is_ok_and(|x| x.is_changed()) {
                        unchanged = false;
                        break;
                    }
                }
            }
            if unchanged {
                let Some(image) = images.get(atlas.image.id()) else {
                    continue;
                };
                let new_dimension = IVec2::new(image.width() as i32, image.height() as i32);
                if output.atlas_dimension == new_dimension {
                    continue;
                }

                let Some(mesh) = get_mesh(&mut mesh2d, &mut mesh3d, &mut meshes) else {
                    continue;
                };

                let Some(VertexAttributeValues::Float32x2(uv0)) =
                    mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
                else {
                    continue;
                };
                for [x, y] in uv0 {
                    *x *= output.atlas_dimension.x as f32 / new_dimension.x as f32;
                    *y *= output.atlas_dimension.y as f32 / new_dimension.y as f32;
                }
                output.atlas_dimension = new_dimension;
                continue;
            }
        }

        let mut buffer = Buffer::new(&mut font_system, Metrics::new(styling.size, styling.size));
        buffer.set_wrap(&mut font_system, Wrap::WordOrGlyph);
        buffer.set_size(&mut font_system, Some(bounds.width), None);
        buffer.set_tab_width(&mut font_system, styling.tab_width);

        buffer.set_rich_text(
            &mut font_system,
            text.segments
                .iter()
                .enumerate()
                .map(|(idx, (text, style))| {
                    (
                        match text {
                            Text3dSegment::String(s) => s.as_str(),
                            Text3dSegment::Extract(e) => segments
                                .get(*e)
                                .map(|x| x.into_inner().as_str())
                                .unwrap_or(""),
                        },
                        style.as_attr().metadata(idx),
                    )
                }),
            Attrs::new()
                .family(Family::Name(&styling.font))
                .style(styling.style)
                .weight(styling.weight),
            Shaping::Advanced,
        );

        buffer.shape_until_scroll(&mut font_system, true);

        let Some(mesh) = get_mesh(&mut mesh2d, &mut mesh3d, &mut meshes) else {
            continue;
        };

        let mut positions = recycle_mesh!(mesh, ATTRIBUTE_POSITION, Float32x3);
        let mut normals = recycle_mesh!(mesh, ATTRIBUTE_NORMAL, Float32x3);
        let mut uv0 = recycle_mesh!(mesh, ATTRIBUTE_UV_0, Float32x2);
        let mut uv1 = recycle_mesh!(mesh, ATTRIBUTE_UV_1, Float32x2);
        let mut colors = recycle_mesh!(mesh, ATTRIBUTE_COLOR, Float32x4);

        let mut indices = if let Some(Indices::U16(indices)) = mesh.remove_indices() {
            indices
        } else {
            Vec::new()
        };
        indices.clear();

        let mut width = 0.0f32;
        let mut sum_width = 0.0f32;

        let mut idx = 0;
        let mut real_index = 0;

        let mut tess_commands = CommandEncoder::default();
        for run in buffer.layout_runs() {
            width = width.max(run.line_w);
            for glyph in run.glyphs.iter() {
                let Some((_, attrs)) = text.segments.get(glyph.metadata) else {
                    continue;
                };
                let dx = -run.line_w * styling.align.as_fac();

                let fills = attrs.fill.unwrap_or(styling.fill);
                let stroke = attrs.stroke.or(styling.stroke);

                let renders: &[_] = match (fills, stroke) {
                    (true, None) => &[(None, attrs.fill_color.unwrap_or(styling.color), 0.)],
                    (false, Some(stroke)) => &[(
                        Some(stroke),
                        attrs.stroke_color.unwrap_or(styling.stroke_color),
                        0.,
                    )],
                    (true, Some(stroke)) => &[
                        (
                            None,
                            attrs.fill_color.unwrap_or(styling.color),
                            (-styling.stroke_offset).max(0.),
                        ),
                        (
                            Some(stroke),
                            attrs.stroke_color.unwrap_or(styling.stroke_color),
                            styling.stroke_offset.max(0.),
                        ),
                    ],
                    (false, None) => &[],
                };

                for (stroke, color, z) in renders.iter().copied() {
                    let Some((pixel_rect, base)) = font_system
                        .db()
                        .with_face_data(glyph.font_id, |file, _| {
                            let Ok(face) = Face::parse(file, 0) else {
                                return None;
                            };
                            let unit_per_em = face.units_per_em() as f32;
                            let entry = GlyphEntry {
                                font: glyph.font_id,
                                glyph_id: glyph.glyph_id,
                                size: FloatOrd(glyph.font_size),
                                weight: styling.weight,
                                stroke,
                            };
                            tess_commands.commands.clear();
                            face.outline_glyph(GlyphId(glyph.glyph_id), &mut tess_commands)?;
                            let (alpha_map, bb) = if let Some(stroke) = stroke {
                                Mask::new(&tess_commands.commands)
                                    .style(Style::Stroke(Stroke {
                                        width: stroke.get() as f32 * unit_per_em / 100.,
                                        start_cap: Cap::Round,
                                        end_cap: Cap::Round,
                                        ..Default::default()
                                    }))
                                    .transform(Some(Transform::scale(
                                        glyph.font_size / unit_per_em * scale_factor,
                                        glyph.font_size / unit_per_em * scale_factor,
                                    )))
                                    .format(Format::Alpha)
                                    .render()
                            } else {
                                Mask::new(&tess_commands.commands)
                                    .transform(Some(Transform::scale(
                                        glyph.font_size / unit_per_em * scale_factor,
                                        glyph.font_size / unit_per_em * scale_factor,
                                    )))
                                    .format(Format::Alpha)
                                    .render()
                            };
                            let (w, h) = (bb.width as usize, bb.height as usize);
                            let pixel_rect = atlas.cache(
                                &mut images,
                                atlas.image.id(),
                                entry,
                                w,
                                h,
                                |buffer, pitch| {
                                    for x in 0..w {
                                        for y in 0..h {
                                            buffer[y * pitch + x * 4 + 3] = alpha_map[y * w + x]
                                        }
                                    }
                                    IVec2::new(w as i32, h as i32)
                                },
                            );
                            let base = Vec2::new(bb.left as f32, bb.top as f32) / scale_factor;
                            Some((pixel_rect, base))
                        })
                        .flatten()
                    else {
                        continue;
                    };

                    let i = idx as u16 * 4;
                    indices.extend([i, i + 1, i + 2, i + 1, i + 3, i + 2]);
                    idx += 1;

                    let local_x0 = glyph.x + base.x;
                    let local_x1 = local_x0 + pixel_rect.width() / scale_factor;

                    let x0 = local_x0 + dx;
                    let y0 = glyph.y + base.y - run.line_y - styling.size;
                    let x1 = local_x1 + dx;
                    let y1 = y0 + pixel_rect.height() / scale_factor;
                    positions.extend([[x0, y0, z], [x1, y0, z], [x0, y1, z], [x1, y1, z]]);

                    normals.extend([[0., 0., 1.]; 4]);

                    // First we cache the pixel position since the texture may be resized.
                    uv0.extend(corners(pixel_rect));

                    let mut uv1_buffer = [[0., 0.], [0., 0.], [0., 0.], [0., 0.]];

                    for (meta_type, i) in [(styling.uv1.0, 0), (styling.uv1.1, 1)] {
                        match meta_type {
                            GlyphMeta::Index => {
                                for pair in &mut uv1_buffer {
                                    pair[i] = real_index as f32;
                                }
                            }
                            GlyphMeta::Advance => {
                                let x0 = (local_x0 + sum_width) / styling.size;
                                let x1 = (local_x0 + sum_width) / styling.size;
                                uv1_buffer[0][i] = x0;
                                uv1_buffer[1][i] = x1;
                                uv1_buffer[2][i] = x0;
                                uv1_buffer[3][i] = x1;
                            }
                            GlyphMeta::PerGlyphAdvance => {
                                let x = (glyph.x
                                    + base.x
                                    + pixel_rect.width() / scale_factor / 2.0
                                    + sum_width)
                                    / styling.size;
                                uv1_buffer[0][i] = x;
                                uv1_buffer[1][i] = x;
                                uv1_buffer[2][i] = x;
                                uv1_buffer[3][i] = x;
                            }
                            GlyphMeta::MagicNumber => {
                                uv1_buffer[0][i] = attrs.magic_number.unwrap_or(0.);
                                uv1_buffer[1][i] = attrs.magic_number.unwrap_or(0.);
                                uv1_buffer[2][i] = attrs.magic_number.unwrap_or(0.);
                                uv1_buffer[3][i] = attrs.magic_number.unwrap_or(0.);
                            }
                            GlyphMeta::RowX => (),
                            GlyphMeta::ColY => (),
                        }
                    }

                    uv1.extend(uv1_buffer);

                    colors.extend([LinearRgba::from(color).to_f32_array(); 4]);
                }
                real_index += 1;
            }
            sum_width += run.line_w;
        }
        let Some(image) = images.get(atlas.image.id()) else {
            continue;
        };

        let (dimension, offset, bb_min) =
            center_aabb_on_anchor(&positions, styling.anchor.as_vec());

        for (meta_type, i) in [(styling.uv1.0, 0), (styling.uv1.1, 1)] {
            match meta_type {
                GlyphMeta::RowX => {
                    for (uv1, position) in uv1.iter_mut().zip(positions.iter()) {
                        uv1[i] = (position[0] - bb_min.x) / dimension.x;
                    }
                }
                GlyphMeta::ColY => {
                    for (uv1, position) in uv1.iter_mut().zip(positions.iter()) {
                        uv1[i] = (position[1] - bb_min.y) / dimension.y;
                    }
                }
                _ => (),
            }
        }

        positions.iter_mut().for_each(|[x, y, _]| {
            *x += offset.x;
            *y += offset.y;
        });

        output.dimension = dimension;
        output.atlas_dimension = IVec2::new(image.width() as i32, image.height() as i32);

        let inv_width = 1.0 / image.width() as f32;
        let inv_height = 1.0 / image.height() as f32;

        uv0.iter_mut().for_each(|[x, y]| {
            *x *= inv_width;
            *y *= inv_height;
        });

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv0);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv1);
        mesh.insert_indices(Indices::U16(indices));
    }
}

#[derive(Debug, Default)]
struct CommandEncoder {
    commands: Vec<Command>,
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
