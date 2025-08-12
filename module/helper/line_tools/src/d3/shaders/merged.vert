#version 300 es
precision highp float;

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;

uniform mat4 u_world_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;

uniform vec2 u_resolution;
uniform float u_width;

void main() 
{
  vec4 viewA = u_view_matrix * u_world_matrix * vec4( inPointA, 1.0 );
  vec4 viewB = u_view_matrix * u_world_matrix * vec4( inPointB, 1.0 );

  vec4 clipA = u_projection_matrix * viewA;
  vec4 clipB = u_projection_matrix * viewB;

  vec2 screenA = ( 0.5 * clipA.xy / clipA.w + 0.5 ) * u_resolution;
  vec2 screenB = ( 0.5 * clipB.xy / clipB.w + 0.5 ) * u_resolution;

  vec2 xBasis = normalize( screenB - screenA );
  vec2 yBasis = vec2( -xBasis.y, xBasis.x );

  vec2 offsetA = screenA + ( xBasis * position.x + yBasis * position.y ) * u_width;
  vec2 offsetB = screenB + ( xBasis * position.x + yBasis * position.y ) * u_width;

  vec2 point = mix( offsetA, offsetB, position.z );
  vec4 clip = mix( clipA, clipB, position.z );

  gl_Position =  vec4( clip.w * ( 2.0 * point / u_resolution - 1.0 ), clip.z, clip.w );
}