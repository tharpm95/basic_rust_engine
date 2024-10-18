struct Uniforms {
    view_proj: mat4x4<f32>;
    model: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> uniforms: Uniforms;

struct VertexInput {
    [[location(0)]] position: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let mvp = uniforms.view_proj * uniforms.model;
    output.position = mvp * input.position;
    return output;
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}