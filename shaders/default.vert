#version 450

layout (location = 0) in vec4 i_pos;
layout (location = 1) in vec4 i_color;

layout (location = 0) out vec4 uFragColor;

layout(push_constant) uniform Push {
  mat4 model_matrix;
  mat3 normal_matrix;
} push;

void main() {
    gl_Position = i_pos * push.model_matrix;
    uFragColor = i_color;
}
