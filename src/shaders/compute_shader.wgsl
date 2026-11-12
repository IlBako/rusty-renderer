@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8,8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let bounds = textureDimensions(output_texture);

    if (global_id.x >= bounds.x) || (global_id.y >= bounds.y) {
        return;
    }

    let ar = f32(bounds.x) / f32(bounds.y);

    let r = (f32(global_id.x) + 0.5) / f32(bounds.x);
    let g = (f32(global_id.y) + 0.5) / f32(bounds.y);
    var color = vec4<f32>(r, g, 0.0, 1.0);

    let center = vec2<f32>(0.5, 0.5);
    var delta = vec2<f32>(r, g) - center;
    delta.x *= ar;

    let radius_sq = 0.25 * 0.25;
    if (dot(delta, delta) <= radius_sq) {
        color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
    }

    textureStore(output_texture, global_id.xy, color);
}