struct VertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) col: vec4<f32>,
    @location(3) index: u32,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,

    @location(0) uv: vec2<f32>,
    @location(1) col: vec4<f32>,
    @location(2) index: u32,
};

struct Camera {
    view: mat4x4<f32>,
};

@group(0) @binding(0)
var textures: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.uv = vertex.uv;
    out.col = vertex.col;
    out.index = vertex.index;

    out.pos = camera.view * vec4<f32>(vertex.pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(fragment: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(
        textures[fragment.index],
        texture_sampler,
        fragment.uv
    ) * fragment.col;
}
