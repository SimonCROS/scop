#version 450

layout (location = 0) in vec4 i_pos;
layout (location = 1) in vec4 i_color;

layout (location = 0) out vec4 uFragColor;

layout(set = 0, binding = 0) uniform CameraBuffer {
    mat4 view;
    mat4 proj;
    mat4 viewproj;
} cameraData;

layout(push_constant) uniform constants {
    mat4 model_matrix;
    mat3 normal_matrix;
} PushConstants;

void main() {
    mat4 transformMatrix = cameraData.view * cameraData.proj * PushConstants.model_matrix;
    gl_Position = i_pos * transformMatrix;
    uFragColor = i_color;
}
