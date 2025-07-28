#version 300 es
precision highp float;

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec2 inPointA;
layout( location = 2 ) in vec2 inPointB;

uniform mat3 u_world_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB, 1.0 ) ).xy;

  vec2 xBasis = normalize( pointB - pointA );
  vec2 yBasis = vec2( -xBasis.y, xBasis.x );

  vec2 offsetA = pointA + ( xBasis * position.x + yBasis * position.y ) * u_width;
  vec2 offsetB = pointB + ( xBasis * position.x + yBasis * position.y ) * u_width;

  vec2 point = mix( offsetA, offsetB, position.z );
  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}