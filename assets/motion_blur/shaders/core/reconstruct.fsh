#version 330
#include <motion_blur:params.glsl>

uniform sampler2D Scene;
uniform sampler2D Motion;
uniform sampler2D NeighborTiles;

out vec4 fragColor;

// Reconstruction filter after McGuire et al. (2012), with Jimenez's alternating sample directions
// and interleaved gradient noise (2014) and Guertin's tile-boundary jitter (2014).

float noise(vec2 pixel, uint frame_index) {
    vec2 p = pixel + vec2(5.588238 * float(frame_index & 63u));
    return fract(52.9829189 * fract(dot(p, vec2(0.06711056, 0.00583715))));
}

// 1 where a lies in front of b, fading out over a short distance behind it.
float in_front(float a, float b) {
    return clamp(1.0 - (a - b) / (0.1 + 0.02 * min(a, b)), 0.0, 1.0);
}

float cone(float gap, float inv_radius) {
    return clamp(1.0 - gap * inv_radius, 0.0, 1.0);
}

float cylinder(float gap, float radius) {
    return 1.0 - smoothstep(0.95 * radius, 1.05 * radius, gap);
}

float sample_weight(float other_z, float other_len, float own_z, float inv_own_len, float gap, float own_cyl) {
    float inv_other_len = 1.0 / max(other_len, 1e-3);
    return in_front(other_z, own_z) * cone(gap, inv_other_len)
        + in_front(own_z, other_z) * cone(gap, inv_own_len)
        + cylinder(gap, other_len) * own_cyl * 2.0;
}

// Approximate linear light when the frame is non-sRGB, so highlights don't darken when blurred.
vec3 to_linear(vec3 c) {
    return linearize != 0u ? c * c : c;
}

vec3 from_linear(vec3 c) {
    return linearize != 0u ? sqrt(max(c, vec3(0.0))) : c;
}

vec3 scene_at(vec2 pixel) {
    return to_linear(texture(Scene, pixel / size).rgb);
}

bool inside(vec2 at) {
    return all(greaterThanEqual(at, vec2(0.0))) && all(lessThan(at, size));
}

void main() {
    vec2 pixel = gl_FragCoord.xy;
    vec4 centre = texture(Scene, pixel / size);

    // Jitter the tile lookup slightly to hide tile-boundary seams.
    float n = noise(pixel, frame);
    float n2 = fract(n * 1.6180339 + 0.5);
    vec2 tile_f = pixel / float(tile_size) + (vec2(n, n2) - 0.5) * 0.5;
    ivec2 last_tile = textureSize(NeighborTiles, 0) - 1;
    vec4 tile_info = texelFetch(NeighborTiles, clamp(ivec2(tile_f), ivec2(0), last_tile), 0);
    vec2 reach = tile_info.xy;
    float reach_length = length(reach);
    // Nothing nearby moves more than half a pixel: most of a still screen stops here.
    if (reach_length < 0.5) {
        fragColor = centre;
        return;
    }
    float fade = smoothstep(0.5, 1.5, reach_length);

    uint max_pairs = max(samples / 2u, 1u);
    uint pairs = clamp(uint(ceil(reach_length * 0.25)), 2u, max_pairs);
    float inv_pairs = 1.0 / float(pairs);

    vec4 own = texelFetch(Motion, ivec2(pixel), 0);
    vec2 own_vel = own.xy;
    vec3 centre_lin = to_linear(centre.rgb);

    // Fast path: the whole neighbourhood moves with nearly one velocity.
    if (tile_info.z < max(1.0, reach_length * 0.25)) {
        vec3 sum = centre_lin;
        float total = 1.0;
        for (uint i = 0u; i < pairs; i++) {
            float t = (float(i) + n) * inv_pairs;
            float w = 1.0 - 0.35 * t * t;
            vec2 offset = own_vel * t;
            if (inside(pixel + offset)) {
                sum += scene_at(pixel + offset) * w;
                total += w;
            }
            if (inside(pixel - offset)) {
                sum += scene_at(pixel - offset) * w;
                total += w;
            }
        }
        fragColor = vec4(mix(centre.rgb, from_linear(sum / total), fade), centre.a);
        return;
    }

    // Full depth-aware reconstruction for silhouettes and parallax.
    ivec2 last = ivec2(size) - 1;
    float own_z = own.z;
    float own_length = length(own_vel);
    float inv_own_len = 1.0 / max(own_length, 1e-3);
    vec2 own_direction = reach;
    if (own_length > 0.5) {
        own_direction = own_vel * (reach_length / own_length);
    }
    float centre_weight = 1.0 / max(own_length, 0.5);
    vec3 sum = centre_lin * centre_weight;
    float total = centre_weight;

    for (uint i = 0u; i < pairs; i++) {
        float t = (float(i) + n) * inv_pairs;
        vec2 direction = (i & 1u) == 1u ? own_direction : reach;
        vec2 offset = direction * t;
        float gap = t * reach_length;
        float own_cyl = cylinder(gap, own_length);

        vec2 at_pos = pixel + offset;
        if (inside(at_pos)) {
            vec4 other = texelFetch(Motion, clamp(ivec2(at_pos), ivec2(0), last), 0);
            float w = sample_weight(other.z, length(other.xy), own_z, inv_own_len, gap, own_cyl);
            sum += scene_at(at_pos) * w;
            total += w;
        }
        vec2 at_neg = pixel - offset;
        if (inside(at_neg)) {
            vec4 other = texelFetch(Motion, clamp(ivec2(at_neg), ivec2(0), last), 0);
            float w = sample_weight(other.z, length(other.xy), own_z, inv_own_len, gap, own_cyl);
            sum += scene_at(at_neg) * w;
            total += w;
        }
    }
    fragColor = vec4(mix(centre.rgb, from_linear(sum / total), fade), centre.a);
}
