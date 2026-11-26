// One frame's cameras and blur settings; `BlurParams` in `motion-blur-core` lays it out (std140).
layout(std140) uniform Params {
    mat4 view_proj;
    mat4 reproject;
    mat4 previous_view_proj;
    vec4 inv_view_proj_row3;
    vec2 size;
    vec2 depth_size;
    float exposure;
    float max_radius;
    uint tile_size;
    uint samples;
    uint frame;
    uint linearize;
    uvec2 pad;
};

// Further than anything in the world: the sky. Motion is kept in 16-bit floats.
const float SKY = 60000.0;

// A frame's motion in pixels, as the blur radius: half the streak the shutter would see, no
// longer than the longest the player allows.
vec2 blur_radius(vec2 motion_px) {
    vec2 radius = motion_px * (0.5 * exposure);
    float len = length(radius);
    if (len > max_radius) {
        return radius * (max_radius / len);
    }
    return radius;
}

vec2 to_pixel(vec4 clip) {
    vec2 ndc = clip.xy / clip.w;
    return (ndc * 0.5 + 0.5) * size;
}

vec2 longer(vec2 a, vec2 b) {
    return dot(b, b) > dot(a, a) ? b : a;
}
