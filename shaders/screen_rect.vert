out vec2 vs_texcoord;

const vec2 RECT_IDX_TO_NDC[4] = vec2[4](
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0)
);

const vec2 RECT_IDX_TO_UV[4] = vec2[4](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0)
);

void main() {
    vs_texcoord = RECT_IDX_TO_UV[gl_VertexID];
    gl_Position = vec4(RECT_IDX_TO_NDC[gl_VertexID], 0.0, 1.0);
}

