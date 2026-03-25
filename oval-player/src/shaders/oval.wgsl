// Oval Video Player — Texture-based egg with dynamic specular
//
// The egg PNG provides shape (alpha mask) and surface appearance.
// Shader adds dynamic specular highlight that tracks the mouse
// and a subtle animated pulse for life.

struct Uniforms {
    oval_size: vec2<f32>,
    mouse_pos: vec2<f32>,
    time: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(1) @binding(0) var egg_texture: texture_2d<f32>;
@group(1) @binding(1) var egg_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    var out: VertexOutput;
    out.position = vec4<f32>(pos[idx], 0.0, 1.0);
    out.uv = pos[idx] * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

const PI: f32 = 3.14159265;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Sample egg texture — RGB is the surface, A is the shape mask
    let tex = textureSample(egg_texture, egg_sampler, uv);

    if (tex.a < 0.01) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    var color = tex.rgb;

    // --- Dynamic specular highlight (follows mouse) ---
    let highlight_center = vec2<f32>(
        0.5 + u.mouse_pos.x * 0.12,
        0.35 + u.mouse_pos.y * 0.06,
    );
    let highlight_d = (uv - highlight_center) / vec2<f32>(0.32, 0.16);
    let highlight_dist = length(highlight_d);
    let highlight = exp(-highlight_dist * highlight_dist * 2.5);
    color = color + vec3<f32>(1.0) * highlight * 0.3;

    // --- Secondary reflection (lower region) ---
    let bottom_center = vec2<f32>(0.5 - u.mouse_pos.x * 0.06, 0.78);
    let bottom_d = (uv - bottom_center) / vec2<f32>(0.22, 0.08);
    let bottom_highlight = exp(-length(bottom_d) * length(bottom_d) * 3.0);
    color = color + vec3<f32>(1.0) * bottom_highlight * 0.08;

    // --- Subtle animated pulse (idle life) ---
    let pulse = 0.5 + 0.5 * sin(u.time * 0.6);
    let drift_center = vec2<f32>(
        0.5 + sin(u.time * 0.25) * 0.03,
        0.38 + cos(u.time * 0.35) * 0.02,
    );
    let drift_d = (uv - drift_center) / vec2<f32>(0.35, 0.18);
    let drift = exp(-length(drift_d) * length(drift_d) * 3.0) * pulse * 0.05;
    color = color + vec3<f32>(drift);

    color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));

    // Premultiplied alpha output
    return vec4<f32>(color * tex.a, tex.a);
}
