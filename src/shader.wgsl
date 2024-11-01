struct Uniforms {
    view_proj: mat4x4<f32>;
    model: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> uniforms: Uniforms;
[[group(0), binding(1)]] var texture: texture_2d<f32>;
[[group(0), binding(2)]] var texture_sampler: sampler;

struct VertexInput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mvp = uniforms.view_proj * uniforms.model;
    output.position = mvp * input.position;
    output.tex_coords = input.tex_coords;
    return output;
}

[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    let sampled_color = textureSample(texture, texture_sampler, input.tex_coords);
    
    // Output the sampled color
    return vec4<f32>(sampled_color.rgb, sampled_color.a); // Include alpha handling
}
