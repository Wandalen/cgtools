#version 300 es

precision mediump float;

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec2 uv;

uniform mat4x4 worldMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec2 vUvs;
out vec3 vWorldNormal;
out vec3 vWorldPosition;
out vec3 vViewPosition;

void main()
{
  vec4 worldPos = worldMatrix * vec4( position, 1.0 );
  vec4 viewPos = viewMatrix * worldPos;

  vUvs = uv;
  vWorldNormal = normalize( mat3x3( worldMatrix ) * normal );
  vWorldPosition = worldPos.xyz;
  vViewPosition = viewPos.xyz;
  gl_Position = projectionMatrix * viewPos;
}
