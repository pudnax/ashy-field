#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 position_offset;
layout (location = 2) in vec3 color;

layout (location = 0) out vec4 f_color;

void main() {
	gl_PointSize = 10.0;
    gl_Position = vec4(position + position_offset, 1.0);
	f_color = vec4(color, 1.0);
}
