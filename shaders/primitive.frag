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
        vec2 uv = vs_texcoord;
        vec4 tex_color = texture(u_tex, uv); 

        if (u_is_font == UTRUE) {
            color.a *= tex_color.a;
        } else {
            color *= tex_color;
        }
    }

    // color.r = vs_texcoord.x;
    // color.g = vs_texcoord.y;
    // color.b = 0.0;
    fs_color = color;
}

