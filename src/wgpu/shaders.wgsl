// ====================
// QUAD SHADER
// ====================

struct QuadVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}
 struct QuadVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn quad_vs_main(in: QuadVertexInput) -> QuadVertexOutput {
    var out: QuadVertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn quad_fs_main(in: QuadVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}
