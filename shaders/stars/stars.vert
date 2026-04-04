#version 440
layout (location = 0) out vec4 v_color;

struct StarData {
	vec3 position;
	float ci;
	float magnitude;
};

layout(binding = 0, std140) readonly buffer StarBuffer {
	StarData stars[];
};

layout(set = 1, binding = 0) uniform PushConstants {
	mat4 projection_matrix;
};


// https://en.wikipedia.org/wiki/Color_index
float ciToTemperature(const in float ci) {
	float comp1 = 1 / (0.92*ci + 1.7);
	float comp2 = 1 / (0.92*ci + 0.62);

	return (comp1 + comp2) * 4600;
}

// https://help.pixera.one/en_US/glsl-effects/colortemperatureglsl
vec3 colorTemperatureToRGB(const in float temperature) {
  // Values from: http://blenderartists.org/forum/showthread.php?270332-OSL-Goodness&p=2268693&viewfull=1#post2268693   
  mat3 m = (temperature <= 6500.0) ? mat3(vec3(0.0, -2902.1955373783176, -8257.7997278925690),
	  vec3(0.0, 1669.5803561666639, 2575.2827530017594),
	  vec3(1.0, 1.3302673723350029, 1.8993753891711275)) : 
	 	mat3(vec3(1745.0425298314172, 1216.6168361476490, -8257.7997278925690),
   	vec3(-2666.3474220535695, -2173.1012343082230, 2575.2827530017594),
	  vec3(0.55995389139931482, 0.70381203140554553, 1.8993753891711275)); 
  return mix(clamp(vec3(m[0] / (vec3(clamp(temperature, 1000.0, 40000.0)) + m[1]) + m[2]), vec3(0.0), vec3(1.0)), vec3(1.0), smoothstep(1000.0, 0.0, temperature));
}

void main(void) {
	if(gl_VertexIndex == 0) {
		gl_Position = vec4(-1.f, -1.f, 0.f, 1.f);
	} else if(gl_VertexIndex == 1) {
		gl_Position = vec4(1.0f, -1.0f, 0.f, 1.f);
	} else if(gl_VertexIndex == 2) {
		gl_Position = vec4(0.0f, 1.0f, 0.f, 1.f);
	}

	StarData star = stars[gl_VertexIndex];

	float temp = ciToTemperature(star.ci);

	v_color = vec4(colorTemperatureToRGB(temp), 1.f);
	// v_color = vec4(star.ci, star.ci, star.ci, 0.f);

}
