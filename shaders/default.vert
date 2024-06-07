#version 450

layout (location = 0) in vec3 i_pos;
layout (location = 1) in vec3 i_color;
layout (location = 2) in vec3 i_normal;
layout (location = 3) in vec2 i_uv;

layout (location = 0) out vec3 o_color;
layout (location = 1) out vec2 o_uv;

layout(set = 0, binding = 0) uniform Camera {
    mat4 projection;
    mat4 view;
} camera;

layout(push_constant) uniform Push {
    mat4 model_matrix;
    mat3 normal_matrix;
} push;

void main() {
    vec4 position_world = push.model_matrix * vec4(i_pos, 1.0);
    gl_Position = camera.projection * camera.view * position_world;
    o_color = i_color;
    o_uv = i_uv;
}
