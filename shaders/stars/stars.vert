#version 440
layout (location = 0) out vec4 v_color;

struct StarData {
	vec3 position;
	float temperature;
	float magnitude;
};

layout(binding = 0, std430) readonly buffer star_data {
	StarData stars[];
};

void main(void) {
	// if(gl_VertexIndex == 0) {
	// 	gl_Position = vec4(-1.f, -1.f, 0.f, 1.f);
	// 	v_color = vec4(1.f, 0.f, 0.f, 1.f);
	// } else if(gl_VertexIndex == 1) {
	// 	gl_Position = vec4(1.0f, -1.0f, 0.f, 1.f);
	// 	v_color = vec4(0.0f, 1.0f, 0.f, 1.f);
	// } else if(gl_VertexIndex == 2) {
	// 	gl_Position = vec4(0.0f, 1.0f, 0.f, 1.f);
	// 	v_color = vec4(0.0f, 0.0f, 1.f, 1.f);
	// }

	uint vertex_index = gl_VertexIndex;	
	v_color = vec4(stars[vertex_index].position, 1.0f);
	gl_Position = vec4(stars[vertex_index].position, 1.f);
}
