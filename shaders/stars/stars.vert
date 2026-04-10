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

layout(set = 1, binding = 0, std140) uniform PushConstants {
	mat4 projection_matrix;
	vec3 cameraPos;
	vec3 camera_rotation;
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

// https://github.com/vhspace/sdl3-rs/blob/master/examples/shaders/gpu-cube.vert
mat4 ortho(float left, float right, float bottom, float top, float near, float far) {
    return mat4(
        2.0 / (right - left), 0, 0, 0,
        0, 2.0 / (top - bottom), 0, 0,
        0, 0, -1.0 / (far - near), 0,
        -(right + left) / (right - left), -(top + bottom) / (top - bottom), -near / (far - near), 1
    );
}

// https://moonside.games/posts/sdl-gpu-sprite-batcher/
const uint[6] triangleIndices = {0, 1, 2, 3, 2, 1};
const vec2 vertexPos[4] = {
    {0.0f, 0.0f},
    {1.0f, 0.0f},
    {0.0f, 1.0f},
    {1.0f, 1.0f}
};

mat3x3 three_dimensional_rotation(float rx, float ry, float rz) {
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

void main(void) {
	// instancing
	uint spriteIndex = gl_VertexIndex / 6;
	StarData star = stars[spriteIndex];

	vec3 starPos = star.position;

	// generation
	uint vert = triangleIndices[gl_VertexIndex % 6];
	vec2 squareVert = vertexPos[vert];
	squareVert *= 0.05f;

	// positioning
	starPos += cameraPos;
	starPos += vec3(squareVert, 1.f);

	// rotation
	vec3 starCoordWithRotation = starPos * three_dimensional_rotation(camera_rotation.x, camera_rotation.y, camera_rotation.z);

	// projection
	gl_Position = vec4(starCoordWithRotation, 1.f) * projection_matrix;

	// coloring
	float dist = sqrt(starPos.x*starPos.x + starPos.y*starPos.y + starPos.z*starPos.z);
	float lightmult = 10.f/(dist);
	float temp = ciToTemperature(star.ci);
	v_color = vec4(colorTemperatureToRGB(temp)*lightmult, 1.f);
}
