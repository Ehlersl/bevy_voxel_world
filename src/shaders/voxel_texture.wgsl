#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    mesh_functions,
    view_transformations::position_world_to_clip
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var atlas_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var atlas_texture_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(102)
var<uniform> atlas_inv_cols: f32;

@group(#{MATERIAL_BIND_GROUP}) @binding(103)
var<uniform> atlas_inv_rows: f32;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,

    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) color: vec4<f32>,

    @location(8) tex_row: u32,
};

struct CustomVertexOutput {
    @builtin(position) position: vec4<f32>,

    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) @interpolate(flat) instance_index: u32,

    @location(8) @interpolate(flat) tex_row: u32,
};

@vertex
fn vertex(vertex: VertexInput) -> CustomVertexOutput {
    var out: CustomVertexOutput;
    var model = mesh_functions::get_world_from_local(vertex.instance_index);

    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index,
    );

    out.world_position = mesh_functions::mesh_position_local_to_world(
        model,
        vec4<f32>(vertex.position, 1.0),
    );

    out.position = position_world_to_clip(out.world_position.xyz);

    out.uv = vertex.uv;

    out.color = vertex.color;
    out.instance_index = vertex.instance_index;
    out.tex_row = vertex.tex_row;

    return out;
}

@fragment
fn fragment(
    in: CustomVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var standard_in: VertexOutput;
    standard_in.position = in.position;
    standard_in.world_normal = in.world_normal;
    standard_in.world_position = in.world_position;
    standard_in.uv = in.uv;
    standard_in.color = in.color;
    standard_in.instance_index = in.instance_index;
    var pbr_input = pbr_input_from_standard_material(standard_in, is_front);

    // determine texture column based on normal
    // 0 = Top, 1 = Bottom, 2 = Side
    var tex_face: u32 = 0u;
    if in.world_normal.y < 0.0 {
        tex_face = 1u;
    } else if in.world_normal.y == 0.0 {
        tex_face = 2u;
    }

    let tile_size = vec2<f32>(atlas_inv_cols, atlas_inv_rows);
    let uv_offset = vec2<f32>(
        f32(tex_face) * tile_size.x,
        f32(in.tex_row) * tile_size.y,
    );

    let atlas_uv = in.uv * tile_size + uv_offset;

    pbr_input.material.base_color = textureSample(
        atlas_texture,
        atlas_texture_sampler,
        atlas_uv,
    );
    pbr_input.material.base_color = pbr_input.material.base_color * in.color;
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material, 
        pbr_input.material.base_color
    );

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}
