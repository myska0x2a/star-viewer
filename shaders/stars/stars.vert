#version 440

layout (location = 0) out vec4 v_color;

layout(set = 1, binding = 0) uniform PushConstants {
    vec4 color;
    float rotation;
    vec2 window_size;
};



void main(void) {
	vec2 pos;
	if(gl_VertexIndex == 0) {
		pos = vec2(-0.5f, -0.5f);
	} else if(gl_VertexIndex == 1) {
		pos = vec2(0.5f, -0.5f);
	} else if(gl_VertexIndex == 2) {
		pos = vec2(0.0f, 0.5f);
	}
	
	mat2 rotate = mat2(cos(rotation), -sin(rotation), sin(rotation), cos(rotation));

	vec2 pos2 = rotate * pos;
	gl_Position = vec4(pos2, 0.0f, 1.0f);

	v_color = color;
}
