in vec4 vs_color;
in vec2 vs_texcoord;

out vec4 fs_color;

uniform sampler2D u_tex;
uniform uint u_use_tex;

const uint UTRUE = uint(1);

void main() {
    vec4 color = vs_color;

    if (u_use_tex == UTRUE) {
        vec4 tex_color = texture(u_tex, vs_texcoord); 
        color.r += tex_color.r;
        color.g += tex_color.g;
        color.b += tex_color.b;
        color.a *= tex_color.a;
    }

    fs_color = color;
}

