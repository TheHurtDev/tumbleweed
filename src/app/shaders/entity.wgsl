struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
}

struct GlobalVertexUniforms {
    perspective_view_proj_matrix: mat4x4<f32>,
    shadow_box_view_proj_matrix: mat4x4<f32>,
}

struct EntityVertexUniforms {
    world_matrix: mat4x4<f32>,
    world_normal_matrix: mat3x3<f32>,
}

struct GlobalFragmentUniforms {
    camera_position: vec3<f32>,
}

struct EntityFragmentUniforms {
    color: vec3<f32>,
}

const BLINN_PHONG_AMBIENT: f32 = 0.15;
const BLINN_PHONG_DIFFUSE: f32 = 0.5;
const BLINN_PHONG_SHININESS: f32 = 16.0;
const BLINN_PHONG_SPECULAR: f32 = 0.3;

const SHADOW_MAP_BIAS: f32 = 0.005;
const SHADOW_MAP_PCF_COUNT: u32 = 3;
const SHADOW_MAP_SIZE: f32 = 2048.0;

const SUNLIGHT_COLOR: vec3<f32> = vec3(1.0, 1.0, 1.0);
const SUNLIGHT_DIRECTION: vec3<f32> = vec3(-0.371391, -0.742781, -0.557086);

@group(0) @binding(0) var<uniform> global_vertex: GlobalVertexUniforms;
@group(1) @binding(0) var<uniform> entity_vertex: EntityVertexUniforms;

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    let world_position = entity_vertex.world_matrix * vec4(input.position, 1.0);
    let clip_position = global_vertex.perspective_view_proj_matrix * world_position;
    let world_normal = normalize(entity_vertex.world_normal_matrix * input.normal);

    return VertexOutput(clip_position, world_position.xyz, world_normal);
}

@group(0) @binding(1) var<uniform> global_fragment: GlobalFragmentUniforms;
@group(1) @binding(1) var<uniform> entity_fragment: EntityFragmentUniforms;

@group(2) @binding(0) var t_shadow_map: texture_depth_2d;
@group(2) @binding(1) var s_shadow_map: sampler_comparison;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let frag_visibility = shadow_visibility_pcf(input.world_position);

    let frag_normal = normalize(input.world_normal);
    let diffuse_amt = frag_visibility * max(dot(frag_normal, -SUNLIGHT_DIRECTION), 0.0);

    let to_camera = normalize(global_fragment.camera_position - input.world_position);
    let halfway = normalize(-SUNLIGHT_DIRECTION + to_camera);
    let specular_amt = frag_visibility * pow(max(dot(frag_normal, halfway), 0.0), BLINN_PHONG_SHININESS);

    let diffuse_color = (BLINN_PHONG_AMBIENT + diffuse_amt * BLINN_PHONG_DIFFUSE) * SUNLIGHT_COLOR * entity_fragment.color;
    let specular_color = specular_amt * BLINN_PHONG_SPECULAR * SUNLIGHT_COLOR;

    return vec4(diffuse_color + specular_color, 1.0);
}

fn shadow_visibility_pcf(world_position: vec3<f32>) -> f32 {
    let light_clip_position = global_vertex.shadow_box_view_proj_matrix * vec4(world_position, 1.0);
    let light_ndc_position = light_clip_position.xyz / light_clip_position.w;

    if (light_ndc_position.z < 0.0 || light_ndc_position.z > 1.0) {
        return 1.0;
    } else {
        var shadow_map_uv_center = 0.5 * light_ndc_position.xy + vec2(0.5);
        shadow_map_uv_center.y = 1.0 - shadow_map_uv_center.y;

        let shadow_map_ref_depth = clamp(light_ndc_position.z - SHADOW_MAP_BIAS, 0.0, 1.0);

        let pcf_size: u32 = 1 + SHADOW_MAP_PCF_COUNT * 2;
        let texel_size = 1.0 / SHADOW_MAP_SIZE;
        var visibility_acc: f32 = 0.0;

        for (var i: u32 = 0; i < pcf_size; i += 1) {
            for (var j: u32 = 0; j < pcf_size; j += 1) {
                let u = shadow_map_uv_center.x + texel_size * (f32(i) - f32(SHADOW_MAP_PCF_COUNT));
                let v = shadow_map_uv_center.y + texel_size * (f32(j) - f32(SHADOW_MAP_PCF_COUNT));

                visibility_acc += textureSampleCompare(
                    t_shadow_map,
                    s_shadow_map,
                    vec2(u, v),
                    shadow_map_ref_depth,
                );
            }
        }

        return visibility_acc / f32(pcf_size * pcf_size);
    }
}

struct ShadowInput {
    @location(0) position: vec3<f32>,
}

struct ShadowOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_shadow(
    input: ShadowInput,
) -> ShadowOutput {
    let clip_position = global_vertex.shadow_box_view_proj_matrix * entity_vertex.world_matrix * vec4(input.position, 1.0);

    return ShadowOutput(clip_position);
}
