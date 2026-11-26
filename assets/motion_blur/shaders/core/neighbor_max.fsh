#version 330
#include <motion_blur:params.glsl>

uniform sampler2D Tiles;

out vec4 fragColor;

// 3x3 neighbourhood max and velocity spread, once per tile instead of per full-resolution pixel.
void main() {
    ivec2 tile = ivec2(gl_FragCoord.xy);
    ivec2 last = textureSize(Tiles, 0) - 1;
    vec2 best = vec2(0.0);
    float max_spread = 0.0;
    vec2 min_b = vec2(1e4);
    vec2 max_b = vec2(-1e4);
    for (int dy = -1; dy <= 1; dy++) {
        for (int dx = -1; dx <= 1; dx++) {
            ivec2 at = clamp(tile + ivec2(dx, dy), ivec2(0), last);
            vec4 t = texelFetch(Tiles, at, 0);
            best = longer(best, t.xy);
            max_spread = max(max_spread, t.z);
            min_b = min(min_b, t.xy);
            max_b = max(max_b, t.xy);
        }
    }
    fragColor = vec4(best, max(max_spread, length(max_b - min_b)), 0.0);
}
