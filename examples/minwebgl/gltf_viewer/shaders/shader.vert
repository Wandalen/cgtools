layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec2 uv_0;
layout( location = 3 ) in vec2 uv_1;
layout( location = 4 ) in vec2 color_0;
layout( location = 5 ) in vec2 color_1;

uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec3 vWorldPos;
out vec3 vNormal;
out vec2 vUv_0;
out vec2 vUv_1;
out vec2 vColor_0;
out vec2 vColor_1;

void main()
{
  vWorldPos = position;
  vNormal = normal;
  vUv_0 = uv_0;
  vUv_1 = uv_1;
  vColor_0 = color_0;
  vColor_1 = color_1;

  gl_Position = projectionMatrix * viewMatrix * vec4( position, 1.0 );
}
