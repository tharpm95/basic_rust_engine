struct Uniforms {
    view_proj: mat4x4<f32>;
    model: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> uniforms: Uniforms;
[[group(0), binding(1)]] var texture: texture_cube<f32>; // Updated to texture_cube
[[group(0), binding(2)]] var texture_sampler: sampler;

struct VertexInput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] tex_coords: vec2<f32>; // Keep as vec2 for input
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coords: vec3<f32>; // Change to vec3
};

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mvp = uniforms.view_proj * uniforms.model;
    output.position = mvp * input.position;
    
    // Convert 2D texture coordinates to 3D direction for cubemap sampling
    output.tex_coords = vec3<f32>(input.tex_coords, 1.0); // Example conversion, adjust as needed
    return output;
}

[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    let sampled_color = textureSample(texture, texture_sampler, input.tex_coords); // Use vec3 for cubemap
    
    // Output the sampled color
    return vec4<f32>(sampled_color.rgb, sampled_color.a); // Include alpha handling
}
