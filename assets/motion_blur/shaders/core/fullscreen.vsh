#version 330

in vec3 Position;

// One oversized triangle that covers the picture: the mod draws three vertices.
void main() {
    gl_Position = vec4(Position.xy, 0.0, 1.0);
}
