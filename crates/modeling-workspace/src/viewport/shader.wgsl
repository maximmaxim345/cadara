struct CameraUniform {
    view_pos: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let pos = vec2<f32>(
        f32(1 - i32(in_vertex_index & 1u) * 2),
        f32(1 - i32(in_vertex_index >> 1u) * 2)
    ) + camera.view_pos;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 1.0, 1.0);
}
