in vec4 vs_color;
in vec2 vs_texcoord;
flat in uint vs_use_tex;

out vec4 fs_color;

uniform sampler2D u_tex;

const uint UTRUE = uint(1);

void main() {
    vec4 color = vs_color;

    if (vs_use_tex == UTRUE) {
        color += texture(u_tex, vs_texcoord);
    }

    fs_color = color;
}

