#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 inPointA;
layout( location = 2 ) in vec2 inPointB;

uniform mat3 u_point_world_matrix;

uniform mat4 u_world_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 pointA = ( u_point_world_matrix * vec3( inPointA, 1.0 ) ).xy;
  vec2 pointB = ( u_point_world_matrix * vec3( inPointB, 1.0 ) ).xy;

  vec2 xBasis = pointB - pointA;
  vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );
  vec2 point = pointA + xBasis * position.x + yBasis * position.y * u_width;
  gl_Position =  u_projection_matrix * u_view_matrix * u_world_matrix * vec4( point, 0.0, 1.0 );
}