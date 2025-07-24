#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 inPointA;
layout( location = 2 ) in vec2 inPointB;
layout( location = 3 ) in vec2 inPointC;

uniform mat3 u_world_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB, 1.0 ) ).xy;
  vec2 pointC = ( u_world_matrix * vec3( inPointC, 1.0 ) ).xy;

  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 miter = vec2( -tangent.y, tangent.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  float sigma = sign( dot( AB + CB, miter ) );

  vec2 ABNorm = normalize( vec2( -AB.y, AB.x ) );
  vec2 CBNorm = normalize( vec2( CB.y, -CB.x ) );

  vec2 p0 = 0.5 * u_width * sigma * ( sigma < 0.0 ? ABNorm : CBNorm );
  vec2 p1 = 0.5 * u_width * sigma * ( sigma < 0.0 ? CBNorm : ABNorm );

  vec2 point = pointB + p0 * position.x + p1 * position.y;

  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}