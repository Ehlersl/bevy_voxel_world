#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput};

@fragment
fn fragment(
    in: VertexOutput,
    @location(8) tex_row: u32,
) -> FragmentOutput {
    var out: FragmentOutput;

    // tex_row will be the array produced by the texture_index_mapper
    // You can use it to send three unsigned integers to the shader
    // based on the voxel type.
    if (tex_row == 1u) {
        out.color = vec4<f32>(1.0, 0.1, 0.1, 1.0);
    }
    if (tex_row == 2u) {
        out.color = vec4<f32>(0.1, 1.0, 0.1, 1.0);
    }
    if (tex_row == 3u) {
        out.color = vec4<f32>(0.1, 0.1, 1.0, 1.0);
    }

    // Multiply by the vertex color to get amient occlusion
    out.color = out.color * in.color;

    return out;
}
