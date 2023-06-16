uniform sampler2D u_tex;
uniform vec4 u_color;

in vec2 vs_texcoord;

out vec4 fs_frag;

void main(void) {
    vec4 color = texture(u_tex, vs_texcoord);
    color += u_color;

    fs_frag = color;
}
