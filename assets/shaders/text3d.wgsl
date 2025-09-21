// Minimal SDF shader used by the Text3d material.
//
// This shader expects the font SDF to be packed into the image alpha channel.
// The SDF generator lives in `src/font.rs` (see `Font::generate`) where a
// grayscale SDF is written into the alpha channel of an RGBA image.

#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 0.0, 1.0),
    );
    out.uv = vertex.uv;
    out.color = vertex.color;
    return out;
}


@group(2) @binding(0) var material_sdf_texture: texture_2d<f32>;
@group(2) @binding(1) var material_sdf_sampler: sampler;

// Convert a normalized SDF value to a smooth alpha using the pixel derivative.
fn contour(d: f32, w: f32) -> f32 {
    return smoothstep(0.5 - w, 0.5 + w, d);
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // Sample SDF stored in alpha channel. The generator writes distance in alpha.
    let sample = textureSample(material_sdf_texture, material_sdf_sampler, mesh.uv);
    let dist = sample.a;

    // Derivative-aware smoothing: width is fwidth(dist) which adapts to
    // transform/scale and provides good anti-aliasing in most cases.
    let width = fwidth(dist);
    let alpha = contour(dist, width);

    // Output glyph color with computed alpha.
    return vec4(mesh.color.rgb, alpha);
}

// Prepass vertex output - includes UV for fragment shader
struct PrepassVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Prepass vertex shader - outputs position and UV for alpha testing
@vertex
fn prepass_vertex(vertex: Vertex) -> PrepassVertexOutput {
    var out: PrepassVertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 0.0, 1.0),
    );
    out.uv = vertex.uv;
    return out;
}

// Prepass fragment shader - performs alpha test to write depth only for visible glyph pixels
@fragment
fn prepass_fragment(
    mesh: PrepassVertexOutput,
) -> @location(0) vec4<f32> {
    // Sample SDF and compute alpha using same logic as main fragment shader
    let sample = textureSample(material_sdf_texture, material_sdf_sampler, mesh.uv);
    let dist = sample.a;
    
    // Use a fixed threshold instead of fwidth() for consistent prepass behavior
    let alpha = contour(dist, 0.02);
    
    // Alpha test - discard pixels that would be transparent
    if alpha < 0.5 {
        discard;
    }
    
    // Return dummy color - only depth matters for prepass
    return vec4(0.0, 0.0, 0.0, 1.0);
}