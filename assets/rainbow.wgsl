
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions,
    pbr_fragment::pbr_input_from_standard_material,
    mesh_view_bindings::globals,
}
#import bevy_render::color_operations::hsv_to_rgb;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool,) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    var out: FragmentOutput;
    out.color = pbr_input.material.base_color;
    out.color *= vec4(hsv_to_rgb(vec3(in.uv_b.x + globals.time, 1.0, 0.5)), 1.0);
    return out;
}