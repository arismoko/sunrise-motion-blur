#version 330
#include <motion_blur:params.glsl>

// The world's depth, reversed (0 is the sky).
uniform sampler2D WorldDepth;

out vec4 fragColor;

// Each pixel's blur radius (rg) and linear view depth in blocks (b), from the world's depth and
// last frame's camera.
void main() {
    vec2 pixel = gl_FragCoord.xy;
    ivec2 texel = ivec2(pixel * depth_size / size);
    texel = clamp(texel, ivec2(0), ivec2(depth_size) - 1);
    float depth = texelFetch(WorldDepth, texel, 0).r;

    vec4 clip_in = vec4(pixel / size * 2.0 - 1.0, depth, 1.0);
    vec4 previous = reproject * clip_in;

    float range = SKY;
    if (depth > 0.0) {
        range = 1.0 / max(dot(inv_view_proj_row3, clip_in), 1e-6);
    }
    if (previous.w <= 0.0) {
        // Behind last frame's camera: no meaningful streak.
        fragColor = vec4(0.0, 0.0, min(range, SKY), 1.0);
        return;
    }
    fragColor = vec4(blur_radius(pixel - to_pixel(previous)), min(range, SKY), 1.0);
}
