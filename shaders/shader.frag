#version 450

layout(location=0) in vec3 in_color;
layout(location=1) in vec3 normal;
layout(location=2) in vec3 worldpos;
layout(location=3) in vec3 camera_direction;

layout(location=0) out vec4 o_color;

const float PI = 3.14159265358979323846264;

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

float distribution(vec3 normal, vec3 halfvector, float roughness) {
	float NdotH = dot(halfvector, normal);
	if (NdotH > 0) {
		float r = roughness*roughness;
		return r / (PI* (1 + NdotH*NdotH*(r-1)) * (1 + NdotH*NdotH*(r-1)));
	} else {
		return 0.0;
	}
}

float geometry(vec3 light, vec3 normal, vec3 view, float roughness) {
	float NdotL = abs(dot(normal, light));
	float NdotV = abs(dot(normal, view));
	return 0.5 / max(0.01, mix(2 * NdotL * NdotV, NdotL + NdotV, roughness));
}

vec3 compute_radiance(vec3 irradiance, vec3 light_direction, vec3 normal,
		vec3 camera_direction, vec3 surface_color) {

	float NdotL = max(dot(normal, light_direction), 0);

	vec3 irradiance_on_surface = irradiance * NdotL;

	float metallic = 1.0;
	float roughness = 0.5;
	roughness = roughness * roughness;

	vec3 F0 = mix(vec3(0.03), surface_color, vec3(metallic));
	vec3 reflected_irradiance = (F0 + (1 - F0) *
			(1 - NdotL)*(1 - NdotL)*(1 - NdotL)*(1 - NdotL)*(1 - NdotL)
			* irradiance_on_surface);
	vec3 refracted_irradiance = irradiance_on_surface - reflected_irradiance;
	vec3 refracted_not_absorbed_irradiance = refracted_irradiance * (1.0 - metallic);

	vec3 halfvector = normalize(0.5 * (camera_direction + light_direction));
	float NdotH = max(dot(normal, halfvector), 0);
	vec3 F = (F0 + (1- F0) *
			(1 - NdotH)*(1 - NdotH)*(1 - NdotH)*(1 - NdotH)*(1 - NdotH));
	vec3 relevant_reflection = reflected_irradiance * F
		* geometry(light_direction, normal, camera_direction, roughness)
		* distribution(normal, halfvector, roughness);

	return refracted_not_absorbed_irradiance * surface_color / PI + relevant_reflection;
}

void main() {
	vec3 L = vec3(0);

	vec3 normal = normalize(normal);

	DirectionalLight dlight = DirectionalLight(
			normalize(vec3(-1.0, -1.0, 0.0)),
			vec3(0.1, 0.1, 0.1));

	L += compute_radiance(dlight.irradiance, dlight.direction_to_light, normal, camera_direction,
			in_color);

	vec3 direction_to_camera = normalize(camera_direction - worldpos);

	for (int i = 0; i < NUMBER_OF_POINTLIGHTS; ++i) {
		PointLight plight = pointlights[i];
		vec3 direction_to_light = normalize(plight.position - worldpos);
		float d = length(worldpos - plight.position);
		vec3 irradiance = plight.luminous_flux / (4*PI*d*d);

		L += compute_radiance(irradiance, direction_to_light, normal, camera_direction,
				in_color);
	};

	o_color = vec4(L / (1.0 + L), 1.0);
}

