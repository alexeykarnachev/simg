uniform sampler2D u_tex;

in vec2 vs_texcoord;

out vec4 fs_frag;

#define PI 3.14159265359

vec2 apply_fish_eye(vec2 p, float strength) {
    vec2 center = vec2(0.5, 0.5);
    vec2 d = p - center;
    float r = sqrt(dot(d, d));
    float power = ( 2.0 * PI / (2.0 * sqrt(dot(center, center))) ) * strength;

    float bind = power > 0.0 ? sqrt(dot(center, center)) : center.y;

    vec2 uv = p;
    if (power > 0.0) {
        uv = center + normalize(d) * tan(r * power) * bind / tan( bind * power);
    } else if (power < 0.0) {
        uv = center + normalize(d) * atan(r * -power * 10.0) * bind / atan(-power * bind * 10.0);
    }

    return uv;
}

void main(void) {
    vec2 uv = vs_texcoord;
    uv = apply_fish_eye(uv, 0.0);

    vec4 color = texture(u_tex, uv);
    // color *= vec4(0.9, 0.9, 0.5, 1.0);

	// float scanline = sin(uv.y * 800.0) * 0.2;
	// color -= scanline;

    fs_frag = color;
}
