#version 440

layout (location = 0) out vec2 out_tex_coord;
layout (location = 1) out vec4 v_color;

struct StarData {
	vec3 position;
	float ci;
	float magnitude;
};

layout(binding = 0, std140) readonly buffer StarBuffer {
	StarData stars[];
};

layout(set = 1, binding = 0, std140) uniform PushConstants {
	mat4 projection_matrix;
	vec3 cameraPos;
	vec3 cameraRot;
};

// https://en.wikipedia.org/wiki/Color_index
float ciToTemperature(const in float ci) {
	float comp1 = 1 / (0.92*ci + 1.7);
	float comp2 = 1 / (0.92*ci + 0.62);

	return (comp1 + comp2) * 4600;
}

// https://help.pixera.one/en_US/glsl-effects/colortemperatureglsl
vec3 colorTemperatureToRGB(const in float temperature) {
  mat3 m = (temperature <= 6500.0) ? mat3(vec3(0.0, -2902.1955373783176, -8257.7997278925690),
	  vec3(0.0, 1669.5803561666639, 2575.2827530017594),
	  vec3(1.0, 1.3302673723350029, 1.8993753891711275)) : 
	 	mat3(vec3(1745.0425298314172, 1216.6168361476490, -8257.7997278925690),
   	vec3(-2666.3474220535695, -2173.1012343082230, 2575.2827530017594),
	  vec3(0.55995389139931482, 0.70381203140554553, 1.8993753891711275)); 
  return mix(clamp(vec3(m[0] / (vec3(clamp(temperature, 1000.0, 40000.0)) + m[1]) + m[2]), vec3(0.0), vec3(1.0)), vec3(1.0), smoothstep(1000.0, 0.0, temperature));
}

// https://github.com/vhspace/sdl3-rs/blob/master/examples/shaders/gpu-cube.vert
mat4 ortho(float left, float right, float bottom, float top, float near, float far) {
    return mat4(
        2.0 / (right - left), 0, 0, 0,
        0, 2.0 / (top - bottom), 0, 0,
        0, 0, -1.0 / (far - near), 0,
        -(right + left) / (right - left), -(top + bottom) / (top - bottom), -near / (far - near), 1
    ); }

// https://moonside.games/posts/sdl-gpu-sprite-batcher/
const uint[6] triangleIndices = {0, 1, 2, 3, 2, 1};
const vec2 vertexPos[4] = { 
    {0.0f, 0.0f},
    {0.5f, 0.0f},
    {0.0f, 0.5f},
    {0.5f, 0.5f}
};

const vec2 textureCoord[4] = {
    {-1.0f, 1.0f},
    {0.f, 1.0f},
    {-1.0f, 0.f},
    {0.f, 0.f}
};

mat3x3 rot3d(float rx, float ry, float rz) {
	mat3x3 x = mat3x3(
		1, 0, 0,
		0, cos(rx), -sin(rx),
		0, sin(rx), cos(rx)
	);
	mat3x3 y = mat3x3(
		cos(ry), 0, sin(ry),
		0, 1, 0,
		-sin(ry), 0, cos(ry)
	);
	mat3x3 z = mat3x3(
		cos(rz), -sin(rz), 0,
		sin(rz), cos(rz), 0,
		0, 0, 1
	);

	return z * y * x;
}

const float ZOOM = 1.f;
const vec2 SCREEN_DIM = vec2(1920.f, 1200.f);
const int TEX_DIMENSIONS = 15;

const float STAR_SIZE_MULT = 0.03f;
const float MAX_STAR_SIZE = 0.02f;
const float MIN_STAR_SIZE = 0.05f;

void main(void) {
	// instancing
	uint spriteIndex = gl_VertexIndex / 6;
	StarData star = stars[spriteIndex];
	
	// view space transform
	vec3 starPos = star.position - cameraPos;
	mat3x3 rotation_matrix = rot3d(cameraRot.x, cameraRot.y, cameraRot.z);
	starPos *= rotation_matrix;

	// star coordinate projection to clip space
	vec4 starPosNDC = vec4(starPos, 1.f) * projection_matrix;

	// finding distance to star
	float dist = sqrt(starPos.x*starPos.x + starPos.y*starPos.y + starPos.z*starPos.z);
	float lightmult = star.magnitude / (dist*dist);
	lightmult = clamp(lightmult, 0.3f, 20.f);

	// billboard vert generation
	uint vert = triangleIndices[gl_VertexIndex % 6];
	vec2 squareVert = vertexPos[vert];
	squareVert *= STAR_SIZE_MULT * lightmult;
	squareVert.x *= (SCREEN_DIM.y / SCREEN_DIM.x);

	// billboard position assignment (within clip space)
	vec4 billboardNDC = starPosNDC + vec4(squareVert * starPosNDC.w, 0.f, 0.f);
	gl_Position = billboardNDC;

	// coloring
	float temp = ciToTemperature(star.ci);
	v_color = vec4(colorTemperatureToRGB(temp)*lightmult, 1.f);

	out_tex_coord = textureCoord[vert];
}
