in vec3 a_position;
in vec3 a_normal;
in vec2 a_texcoord;
in vec4 a_color;
in uint a_flags;

uniform mat4 u_model_mat;
uniform mat4 u_view_mat;
uniform mat4 u_proj_mat;

out vec3 vs_world_pos;
out vec4 vs_color;
out vec3 vs_normal;
out vec2 vs_texcoord;
flat out uint vs_flags;

void main() {
    mat4 mvp_mat = u_proj_mat * u_view_mat * u_model_mat;
    mat3 normal_mat = transpose(inverse(mat3(u_model_mat)));

    vec4 position = vec4(a_position, 1.0);
    vec3 world_position = (u_model_mat * position).xyz;
    vec4 proj_position = mvp_mat * position;
    vec3 normal = normal_mat * a_normal;

    vs_world_pos = world_position;
    vs_color = a_color;
    vs_normal = a_normal;
    vs_texcoord = a_texcoord;
    vs_flags = a_flags;
    gl_Position = proj_position;
}
