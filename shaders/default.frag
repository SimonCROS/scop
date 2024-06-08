#version 450

layout (location = 0) flat in vec3 i_color;
layout (location = 1) in vec2 i_uv;

layout (location = 0) out vec4 o_color;

layout (set = 1, binding = 0) uniform sampler2D texSampler;

layout (push_constant) uniform Push {
    mat4 model_matrix;
    mat3 normal_matrix;
    float flat_texture_interpolation;
} push;

void main() {
    o_color = mix(vec4(i_color, 1.0), texture(texSampler, i_uv), push.flat_texture_interpolation);
}
