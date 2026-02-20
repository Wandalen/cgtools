precision mediump float;

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec2 uv;

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;
uniform mat3x3 normalMatrix;

out vec2 vUvs;
out vec3 vWorldNormal;
out vec3 vWorldPosition;

void main()
{
  vec4 worldPos = worldMatrix * vec4( position, 1.0 );
  vUvs = uv;
  vWorldNormal = normalize( mat3x3( normalMatrix ) * normal );
  vWorldPosition = worldPos.xyz;
  gl_Position = projectionMatrix * viewMatrix * worldPos;
}
