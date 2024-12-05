struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) texcoord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct ShaderCameraUniform {
    viewproj: mat4x4<f32>,
    near: f32,
    far: f32,
    _padding: vec2<f32>,
}

//----------------------------------------------------------------------

@group(1) @binding(0)
var<uniform> camera: ShaderCameraUniform;

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.viewproj * vec4<f32>(vertex.position, 1.0);
    out.uv = vertex.texcoord;
    out.color = vertex.color;
    return out;
}

//----------------------------------------------------------------------

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
    //return vec4<f32>(in.color, 1.0);
}