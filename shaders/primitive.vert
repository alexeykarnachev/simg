in vec3 a_position;
in vec4 a_color;

uniform mat4 u_transform;

out vec4 vs_color;

void main() {
    vec4 position = u_transform * vec4(a_position, 1.0);

    vs_color = a_color;
    gl_Position = position;
}
