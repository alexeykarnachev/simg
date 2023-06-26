const uint UTRUE = uint(1);

in vec4 vs_color;
in vec2 vs_texcoord;
flat in uint vs_has_tex;

out vec4 fs_color;

uniform sampler2D u_tex;

void main() {
    vec4 color = vs_color;

    if (vs_has_tex == UTRUE) {
        vec2 tex_size = vec2(textureSize(u_tex, 0));
        vec2 uv = vs_texcoord;
        uv /= tex_size;

        vec4 tex_color = texture(u_tex, uv); 
        color.r += tex_color.r;
        color.g += tex_color.g;
        color.b += tex_color.b;
        color.a *= tex_color.a;
    }

    color.a = 1.0;

    fs_color = color;
}

