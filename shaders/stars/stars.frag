#version 440

layout (set = 2, binding = 0) uniform sampler2D tex_sampler;

layout (location = 0) in vec2 tex_coord;
layout (location = 1) in vec4 v_color;

layout (location = 0) out vec4 o_frag_color;

// glslc triangle.frag -o triangle.frag.spv
void main() {
	// o_frag_color = v_color;
	// o_frag_color = vec4(1.f, 1.f, 1.f, 1.f);
	vec4 tex = texture(tex_sampler, tex_coord);

	// if (tex.z < 0.0001) {
	// 	discard;
	// }

	// o_frag_color = tex.w * v_color / 1.5f;
	o_frag_color = tex;

}
