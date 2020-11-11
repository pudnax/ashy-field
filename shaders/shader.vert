#version 450

layout (location = 0) in vec4 position;

layout (location = 0) out vec4 f_color;

void main() {
	gl_PointSize = 10.0;
    gl_Position = position;
	f_color = vec4(0.4, 1.0, 0.5, 1.0);
}
