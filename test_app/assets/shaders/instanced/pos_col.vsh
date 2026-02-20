#version 450

layout(set = 0, binding = 0) uniform Camera {
    mat4 viewMat;
    mat4 projMat;
};

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec4 in_color;

layout(location = 2) in vec4 instance_model_0;
layout(location = 3) in vec4 instance_model_1;
layout(location = 4) in vec4 instance_model_2;
layout(location = 5) in vec4 instance_model_3;

layout(location = 0) out vec4 v_color;

void main() {
    mat4 instance_model = mat4(
        instance_model_0,
        instance_model_1,
        instance_model_2,
        instance_model_3
    );
    gl_Position = projMat * viewMat * instance_model * vec4(in_pos, 1.0);
    v_color = in_color;
}