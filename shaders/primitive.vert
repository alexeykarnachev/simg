#version 460 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec4 a_color;

out vec4 vs_color;

void main() {
    vs_color = a_color;
    gl_Position = vec4(a_position, 1.0);
}
