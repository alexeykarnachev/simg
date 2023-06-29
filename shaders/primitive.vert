in vec3 a_position;
in vec3 a_normal;
in vec2 a_texcoord;
in vec4 a_color;
in uint a_has_tex;

uniform mat4 u_position_mat;
uniform mat3 u_normal_mat;

out vec4 vs_color;
out vec3 vs_normal;
out vec2 vs_texcoord;
flat out uint vs_has_tex;

void main() {
    vec4 position = u_position_mat * vec4(a_position, 1.0);
    vec3 normal = u_normal_mat * a_normal;

    vs_color = a_color;
    vs_normal = a_normal;
    vs_texcoord = a_texcoord;
    vs_has_tex = a_has_tex;
    gl_Position = position;
}
