const uint UTRUE = uint(1);

in vec4 vs_color;
in vec2 vs_texcoord;
flat in uint vs_has_tex;

out vec4 fs_color;

uniform sampler2D u_tex;
uniform uint u_is_font;

void main() {
    vec4 color = vs_color;

    if (vs_has_tex == UTRUE) {
        vec2 tex_size = vec2(textureSize(u_tex, 0));
        vec2 uv = vs_texcoord;
        uv /= tex_size;

        vec4 tex_color = texture(u_tex, uv); 

        if (u_is_font == UTRUE) {
            color.a *= tex_color.a;
        } else {
            color *= tex_color;
        }
    }

    fs_color = color;
}

