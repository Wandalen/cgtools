#version 300 es
precision highp float;

uniform mat4x4 projectionMatrix;
uniform mat3x3 worldMatrix;

uniform vec2 position;
uniform float radius;

out vec2 vUv;

const vec2 vertices[] = vec2[]
(
  vec2( 0.0, 1.0 ),
  vec2( 1.0, 1.0 ),
  vec2( 0.0, 0.0 ),
  vec2( 1.0, 0.0 )
);

void main() 
{
  vec2 worldPosition = ( worldMatrix * vec3( position + ( vertices[ gl_VertexID ] * 2.0 - 1.0 ) * radius, 1.0 ) ).xy;
  vUv = vertices[ gl_VertexID ];

  gl_Position = projectionMatrix * vec4( worldPosition, 0.0, 1.0 );
}