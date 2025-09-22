@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(16, 16)
fn convert_main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {
    let dimensions: vec2<u32> = textureDimensions(output_texture);
    let coords = vec2<u32>(global_id.xy);

    if(coords.x >= dimensions.x || coords.y >= dimensions.y) {
        return;
    }

    let color = textureLoad(input_texture, coords.xy, 0);
    textureStore(output_texture, coords.xy, color);
}