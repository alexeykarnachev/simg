#version 300 es
#ifdef GL_ES
precision highp float;
#endif

// #version 460 core

in vec4 vs_color;

out vec4 fs_color;

void main() {
    fs_color = vs_color;
}

