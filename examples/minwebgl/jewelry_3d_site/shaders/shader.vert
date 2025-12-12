#version 300 es

precision mediump float;

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec2 uv;

uniform mat4x4 modelMatrix;
uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

out vec2 vUvs;
out vec3 vPosition;

void main() 
{	
  vec4 worldPos = modelMatrix * vec4( position, 1.0 );

  vUvs = uv;
  vPosition = worldPos.xyz;
  gl_Position = projectionMatrix * viewMatrix * worldPos;
}
