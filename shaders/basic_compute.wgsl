@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // Get the coordinates of the current pixel
   let pixel_x = global_id.x;
   let pixel_y = global_id.y;

   // Get the output texture dimensions
   let dimensions = textureDimensions(output_texture);
   let width = dimensions.x;
   let height = dimensions.y;

    // Check bounds to avoid out-of-bounds writes
    if (pixel_x >= width || pixel_y >= height) {
        return;
    }

    // Compute some color based on the pixel coordinates
    let red = f32(pixel_x) / f32(width);
    let green = f32(pixel_y) / f32(height);
    let blue = 0.5;

    // Write the color to the output texture
    textureStore(
        output_texture,
        vec2<i32>(i32(pixel_x), i32(pixel_y)),
        vec4<f32>(red, green, blue, 1.0)
    );
}