#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec3 color;

uniform mat4x4 projectionMatrix;
uniform mat3x3 worldMatrix;

out vec3 vColor;

void main() 
{
  vColor = color;

  vec2 worldPosition = ( worldMatrix * vec3( position, 1.0 ) ).xy;

  gl_Position = projectionMatrix * vec4( worldPosition, 0.0, 1.0 );
  gl_PointSize = 10.0 + 5.0 * color.g;
}