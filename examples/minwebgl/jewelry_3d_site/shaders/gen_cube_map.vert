#version 300 es

precision mediump float;

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;
layout( location = 2 ) in vec3 uv;

uniform mat4x4 worldMatrix;
uniform mat3x3 normalMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec3 vNormal;
out vec3 vPosition;

void main()
{
  vec4 worldPos = worldMatrix * vec4( position, 1.0 );

  vNormal = normalize( normalMatrix * normal );
  vPosition = worldPos.xyz;
  gl_Position = projectionMatrix * viewMatrix * worldPos;
}
