struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) sample_direction: vec3<f32>,
}

struct VertexUniforms {
    proj_matrix: mat4x4<f32>,
    rotation_matrix: mat3x3<f32>,
}

@group(0) @binding(0) var<uniform> vertex_uniforms: VertexUniforms;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
) -> VertexOutput {
    let clip_position = vertex_uniforms.proj_matrix * vec4(position, 1.0);

    let sample_direction = vertex_uniforms.rotation_matrix * position;

    return VertexOutput(clip_position, sample_direction);
}

@group(1) @binding(0) var t_skybox: texture_cube<f32>;
@group(1) @binding(1) var s_skybox: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample_direction = normalize(input.sample_direction);
    let color = textureSample(t_skybox, s_skybox, sample_direction).rgb;

    return vec4(color, 1.0);
}
