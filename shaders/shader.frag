#version 450

layout(location=0) in vec3 in_color;
layout(location=1) in vec3 normal;
layout(location=2) in vec3 worldpos;

layout(location=0) out vec4 o_color;

struct DirectionalLight {
	vec3 direction_to_light;
	vec3 irradiance;
};

struct PointLight  {
	vec3 position;
	vec3 luminous_flux;
};

const int NUMBER_OF_POINTLIGHTS = 3;

PointLight pointlights [NUMBER_OF_POINTLIGHTS] = {
	PointLight(vec3(1.5,0.0,0.0),vec3(10,10,10)),
	PointLight(vec3(1.5,0.2,0.0),vec3(5,5,5)),
	PointLight(vec3(1.6,-0.2,0.1),vec3(5,5,5))
};


vec3 compute_radiance(vec3 irradiance, vec3 light_direction, vec3 normal, vec3 surface_color) {
	return irradiance * (max(dot(normal, light_direction), 0)) * surface_color;
}

void main() {
	vec3 L = vec3(0);

	DirectionalLight dlight = DirectionalLight(
			normalize(vec3(-1.0, -1.0, 0.0)),
			vec3(0.1, 0.1, 0.1));

	L += compute_radiance(dlight.irradiance, dlight.direction_to_light, normal, in_color);

	const float PI = 3.14159265358979323846264;
	for (int i = 0; i < NUMBER_OF_POINTLIGHTS; ++i) {
		PointLight plight = pointlights[i];
		vec3 direction_to_light = normalize(plight.position - worldpos);
		float d = length(worldpos - plight.position);
		vec3 irradiance = plight.luminous_flux / (4*PI*d*d);

		L += compute_radiance(irradiance, direction_to_light, normal, in_color);
	};

	o_color = vec4(L / (1.0 + L), 1.0);
}

