struct Uniforms {
    x: f32,
    y: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index & 1u) * 2) + uniforms.x;
    let y = f32(1 - i32(in_vertex_index >> 1u) * 2) + uniforms.y;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>(
        f32(in_vertex_index & 1u),
        f32(in_vertex_index >> 1u),
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 1.0, 1.0);
}
