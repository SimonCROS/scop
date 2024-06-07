#version 450

layout (location = 0) in vec3 i_color;
layout (location = 1) in vec2 i_uv;

layout (location = 0) out vec4 o_color;

layout (set = 1, binding = 0) uniform sampler2D texSampler;

void main() {
    o_color = texture(texSampler, i_uv);
    // o_color = vec4(vec2(i_color), 0, 0);
}
