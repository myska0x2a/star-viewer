#version 440
layout (location = 0) in vec4 v_color;
layout (location = 0) out vec4 o_frag_color;

// glslc triangle.frag -o triangle.frag.spv
void main() {
	o_frag_color = v_color;
	// o_frag_color = vec4(1.0f, 0.0f, 0.0f, 1.0f);

}
