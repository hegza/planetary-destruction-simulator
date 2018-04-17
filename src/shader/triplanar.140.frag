#version 140

uniform sampler2D t_vertical;
uniform sampler2D t_horizontal;
uniform mat4 vpmatrix;
in vec3 v_position;
in vec3 v_normal;

out vec4 f_color;

// Triplanar texture-scale
const float TEX_SCALE = 2.;

// Constant lighting
const float AMBIENT_LIGHT = 0.3;
const vec3 LIGHT_DIR = vec3(-1.0, 0.0, 0.0);

// Equatorial material properties, albedo == diffuse
const float ALBEDO_EQ = 0.4;

// Polar material properties, albedo == diffuse
const float ALBEDO_POLAR = 0.8;

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
    vec3 xaxis = texture2D( t_horizontal, v_position.yz * TEX_SCALE).rgb;
    vec3 yaxis = texture2D( t_vertical, v_position.xz * TEX_SCALE).rgb;
    vec3 zaxis = texture2D( t_horizontal, v_position.xy * TEX_SCALE).rgb;

    // Blended color
    vec3 normal_tex = xaxis * blending.x + yaxis * blending.y + zaxis * blending.z;

    // Ambient term
    vec3 ambient_intensity = AMBIENT_LIGHT * normal_tex;

    // Diffuse term
    float lambertian = dot(v_normal, LIGHT_DIR);
    float diffuse_intensity_eq = ALBEDO_EQ * lambertian;
    float diffuse_intensity_polar = ALBEDO_POLAR * lambertian;

    vec3 diffuse_intensity =
        diffuse_intensity_eq * xaxis * blending.x +
        diffuse_intensity_polar * yaxis * blending.y +
        diffuse_intensity_eq * zaxis * blending.z;

    vec3 color = normal_tex;
    f_color = vec4(ambient_intensity + diffuse_intensity, 1.0);
}
