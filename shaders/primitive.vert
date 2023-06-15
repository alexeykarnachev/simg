in vec3 a_position;
in vec2 a_texcoord;
in vec4 a_color;

uniform mat4 u_transform;

out vec4 vs_color;
out vec2 vs_texcoord;

void main() {
    vec4 position = u_transform * vec4(a_position, 1.0);

    vs_color = a_color;
    vs_texcoord = a_texcoord;
    gl_Position = position;
}
