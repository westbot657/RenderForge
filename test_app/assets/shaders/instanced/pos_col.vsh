#version 330 core

uniform mat4 viewMat;
uniform mat4 projMat;

layout(location = 0) in mat4 instance_model;

layout(location = 4) in vec3 in_pos;
layout(location = 5) in vec4 in_color;

out vec4 v_color;

void main() {
    gl_Position = projMat * viewMat * instance_model * vec4(in_pos, 1.0);
    v_color = in_color;
}
