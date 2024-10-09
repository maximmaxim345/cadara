struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
};

struct Vertex {
    @location(0) pos: vec3<f32>,
}

@vertex
fn vs_main(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Transform the vertex position to world space
    let world_pos = vertex.pos;

    // Transform the world position to clip space
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);

    // Pass the world position to the fragment shader
    out.world_position = world_pos;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate the normal using dpdx and dpdy on the world position
    let dx = dpdx(in.world_position);
    let dy = dpdy(in.world_position);
    let normal = normalize(cross(dy, dx));

    // Map the normal to color space
    let color = normal * 0.5 + 0.5;

    return vec4<f32>(color, 1.0);
}
