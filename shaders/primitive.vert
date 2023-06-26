in vec3 a_position;
in vec2 a_texcoord;
in vec4 a_color;
in uint a_has_tex;

uniform mat4 u_transform;

out vec4 vs_color;
out vec2 vs_texcoord;
flat out uint vs_has_tex;

void main() {
    vec4 position = u_transform * vec4(a_position, 1.0);

    vs_color = a_color;
    vs_texcoord = a_texcoord;
    vs_has_tex = a_has_tex;
    gl_Position = position;
}
