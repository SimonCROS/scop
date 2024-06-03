#version 450

layout(location = 0) in vec3 i_color;

layout(location = 0) out vec4 o_color;

layout(binding = 1) uniform sampler2D texSampler;

void main() {
    o_color = texture(texSampler, vec2(i_color));
    // o_color = vec4(vec2(i_color), 0, 0);
}
