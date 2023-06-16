uniform sampler2D u_tex;

in vec2 vs_texcoord;

out vec4 fs_frag;

void main(void) {
    vec4 color = texture(u_tex, vs_texcoord);
    color.r += 0.3;

    fs_frag = color;
}
