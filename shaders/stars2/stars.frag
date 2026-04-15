#version 440

layout (set = 2, binding = 0) uniform sampler2D tex_sampler;

layout (location = 0) in vec2 tex_coord;
layout (location = 1) in vec4 v_color;

layout (location = 0) out vec4 o_frag_color;

// glslc triangle.frag -o triangle.frag.spv
void main() {
	// o_frag_color = v_color;
	o_frag_color = texture(tex_sampler, tex_coord) * v_color;
}
