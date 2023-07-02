const uint UTRUE = uint(1);
const uint UFALSE = uint(0);

in vec3 vs_world_pos;
in vec4 vs_color;
in vec3 vs_normal;
in vec2 vs_texcoord;
flat in uint vs_flags;

out vec4 fs_color;

uniform sampler2D u_tex;
uniform uint u_is_font;
uniform uint u_is_blinn_phong;
uniform float u_shininess;
uniform vec3 u_camera_pos;

const vec3 LIGHT_DIR = vec3(0.0, 0.0, -1.0);
const vec3 LIGHT_COLOR = vec3(0.8, 0.6, 0.4);
const vec3 AMBIENT_COLOR = vec3(0.08, 0.06, 0.04);

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

    if ((vs_flags & HasNormal) != UFALSE && u_is_blinn_phong == UTRUE) {
        vec3 light_dir = normalize(LIGHT_DIR);
        vec3 normal = normalize(vs_normal);
        float lambertian = max(dot(normal, -light_dir), 0.0);

        float specular = 0.0;
        if (lambertian > 0.0) {
            vec3 view_dir = normalize(vs_world_pos - u_camera_pos);
            vec3 half_dir = -normalize(view_dir + light_dir);
            float specular_angle = max(dot(half_dir, normal), 0.0);    
            specular = pow(specular_angle, u_shininess); 
        }
        
        vec3 diffuse_color = color.rgb * lambertian * LIGHT_COLOR;
        vec3 specular_color = LIGHT_COLOR * specular;
        color = vec4(AMBIENT_COLOR + diffuse_color + specular_color, color.a);
    }

    fs_color = color;
}

