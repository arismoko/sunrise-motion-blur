#version 330
#include <motion_blur:params.glsl>

uniform sampler2D Motion;

out vec4 fragColor;

// The longest motion along each row of a tile, and how much the motions in it differ.
void main() {
    int width = int(size.x);
    int first = int(gl_FragCoord.x) * int(tile_size);
    int end = min(first + int(tile_size), width);
    int y = int(gl_FragCoord.y);
    vec2 best = vec2(0.0);
    vec2 min_v = vec2(1e4);
    vec2 max_v = vec2(-1e4);
    for (int x = first; x < end; x++) {
        vec2 v = texelFetch(Motion, ivec2(x, y), 0).xy;
        best = longer(best, v);
        min_v = min(min_v, v);
        max_v = max(max_v, v);
    }
    float spread = first < end ? length(max_v - min_v) : 0.0;
    fragColor = vec4(best, spread, 0.0);
}
