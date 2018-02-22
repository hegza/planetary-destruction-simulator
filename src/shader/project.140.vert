#version 140
uniform mat4 vpmatrix;

// Instance data
uniform vec4 orientation;
uniform vec3 translation;
uniform float scale;

in vec3 position;
in vec3 normal;
// TODO: tex-coords

out vec3 v_position;
out vec3 v_normal;

vec3 qrotate(vec3 v, vec4 q) {
    return v + 2.0*cross(cross(v, q.xyz ) + q.w*v, q.xyz);
}

void main() {
    vec3 pos = qrotate(position * scale, orientation) + translation;

    v_position = pos;
    v_normal = qrotate(normal, orientation);
    gl_Position = vpmatrix * vec4(pos, 1.0);
}
