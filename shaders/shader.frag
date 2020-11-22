#version 450

layout(location=0) in vec4 in_color;
layout(location=1) in vec3 normal;

layout(location=0) out vec4 o_color;

void main() {
	vec3 direction_to_light = normalize(vec3(-1.0, -1.0, 0.0));
	o_color = 0.5 * max(1.0 + dot(normal, direction_to_light), 0) * in_color;
}

