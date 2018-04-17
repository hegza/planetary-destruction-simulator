#version 140

uniform sampler2D t_vertical;
uniform sampler2D t_horizontal;
in vec3 v_position;
in vec3 v_normal;

out vec4 f_color;

// Triplanar texture-scale
const float TEX_SCALE = 2.;

// Constant lighting
const float POLAR_ALBEDO = 1.0;
const float ALBEDO = 0.6;

vec3 triplanar_blend(vec3 world_normal) {
    vec3 blending = abs( world_normal );
    blending = normalize(max(blending, 0.00001));
    float b = (blending.x + blending.y + blending.z);
    blending /= vec3(b, b, b);
    return blending;
}

void main() {

    // Triplanar blending
    vec3 blending = triplanar_blend(v_normal);
    vec3 xaxis = texture2D( t_horizontal, v_position.yz*TEX_SCALE).rgb;
    vec3 yaxis = texture2D( t_vertical, v_position.xz*TEX_SCALE).rgb;
    vec3 zaxis = texture2D( t_horizontal, v_position.xy*TEX_SCALE).rgb;
    vec3 normalTex = xaxis * ALBEDO * blending.x + yaxis * POLAR_ALBEDO * blending.y + zaxis * ALBEDO * blending.z;

    vec3 color = normalTex;
    f_color = vec4(color, 1.0);
}
