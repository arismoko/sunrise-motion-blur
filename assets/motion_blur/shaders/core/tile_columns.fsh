#version 330
#include <motion_blur:params.glsl>

uniform sampler2D TileRows;

out vec4 fragColor;

void main() {
    int height = int(size.y);
    int first = int(gl_FragCoord.y) * int(tile_size);
    int end = min(first + int(tile_size), height);
    int x = int(gl_FragCoord.x);
    vec2 best = vec2(0.0);
    float max_spread = 0.0;
    vec2 min_b = vec2(1e4);
    vec2 max_b = vec2(-1e4);
    for (int y = first; y < end; y++) {
        vec4 row = texelFetch(TileRows, ivec2(x, y), 0);
        best = longer(best, row.xy);
        max_spread = max(max_spread, row.z);
        min_b = min(min_b, row.xy);
        max_b = max(max_b, row.xy);
    }
    float spread = first < end ? max(max_spread, length(max_b - min_b)) : 0.0;
    fragColor = vec4(best, spread, 0.0);
}
