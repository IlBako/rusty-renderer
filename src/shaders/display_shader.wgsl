@group(0) @binding(0)
var output_texture: texture_2d<f32>;


struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Generate UV coordinates for an oversized full-screen triangle: (0,0), (2,0), (0,2)
    let uv = vec2<f32>(
        f32((in_vertex_index << 1u) & 2u),
        f32(in_vertex_index & 2u)
    );

    // Map UVs to clip space
    out.clip_position = vec4<f32>(uv*2.0 - 1.0, 0.0, 1.0);

    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_coords = vec2<i32>(in.clip_position.xy);
    return textureLoad(output_texture, pixel_coords, 0);
}