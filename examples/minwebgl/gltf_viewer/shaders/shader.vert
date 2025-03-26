layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec2 uv;

uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec3 vWorldPos;
out vec3 vNormal;
out vec2 vUv;

void main()
{
  vNormal = normal;
  vUv = uv;
  vWorldPos = position;

  gl_Position = projectionMatrix * viewMatrix * vec4( position, 1.0 );
}
