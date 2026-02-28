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

// ====================
// TEXT SHADER
// ====================

struct TextVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct TextVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0)
var glyph_texture: texture_2d<f32>;

@group(0) @binding(1)
var glyph_sampler: sampler;

@vertex
fn text_vs_main(in: TextVertexInput) -> TextVertexOutput {
    var out: TextVertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    out.color = in.color;
    return out;
}

@fragment
fn text_fs_main(in: TextVertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(glyph_texture, glyph_sampler, in.tex_coords).r;
    return vec4<f32>(in.color.rgb, alpha * in.color.a);
}
