const uint UTRUE = uint(1);

struct Texture {
    sampler2D tex;
    uint is_used;
};

in vec4 vs_color;
in vec2 vs_texcoord;

out vec4 fs_color;

uniform Texture u_tex;

void main() {
    vec4 color = vs_color;

    if (u_tex.is_used == UTRUE) {
        vec2 tex_size = vec2(textureSize(u_tex.tex, 0));
        vec2 uv = vs_texcoord / tex_size;

        vec4 tex_color = texture(u_tex.tex, uv); 
        color.r += tex_color.r;
        color.g += tex_color.g;
        color.b += tex_color.b;
        color.a *= tex_color.a;
    }

    fs_color = color;
}

