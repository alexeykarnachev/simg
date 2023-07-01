const uint UTRUE = uint(1);
const uint UFALSE = uint(0);

in vec4 vs_color;
in vec3 vs_normal;
in vec2 vs_texcoord;
flat in uint vs_flags;

out vec4 fs_color;

uniform sampler2D u_tex;
uniform uint u_is_font;

void main() {
    vec4 color = vs_color;

    if ((vs_flags & HasTexture) != UFALSE) {
        vec2 uv = vs_texcoord;
        vec4 tex_color = texture(u_tex, uv); 

        if (u_is_font == UTRUE) {
            color.a *= tex_color.a;
        } else {
            color *= tex_color;
        }
    }

    if ((vs_flags & HasNormal) != UFALSE) {
        float k = dot(normalize(vs_normal), vec3(0.0, 1.0, 0.0));
        k = max(0.0, k);
        color = vec4(color.rgb * k, color.a);
    }

    fs_color = color;
}

