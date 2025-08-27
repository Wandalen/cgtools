#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;

uniform mat3 u_world_matrix;
uniform mat3 u_view_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

out vec2 vUv;

void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA.xy, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB.xy, 1.0 ) ).xy;

  vec2 xBasis = normalize( pointA - pointB );
  vec2 yBasis = vec2( -xBasis.y, xBasis.x );
  vec2 point = pointA + xBasis * position.x * u_width + yBasis * position.y * u_width;

  vUv.y = step( 0.0, float( position.y ) );
  vUv.y = mix( 1.0 - vUv.y, vUv.y, float( gl_InstanceID ) );
  vUv.x = mix( 0.0, 1.0, float( gl_InstanceID ) );

  vec3 view_point = u_view_matrix * vec3( point, 1.0 );

  gl_Position =  u_projection_matrix * vec4( view_point.xy, 0.0, 1.0 );
}