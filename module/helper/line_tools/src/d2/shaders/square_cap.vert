#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;

uniform vec2 u_pointA;
uniform vec2 u_pointB;

uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 xBasis = normalize( u_pointB - u_pointA );
  vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );
  vec2 point = u_pointB + 0.5 * xBasis * position.x * u_width + yBasis * position.y * u_width;
  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}