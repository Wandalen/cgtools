#version 300 es
precision highp float;

const float PI = 3.1415926;

layout( location = 0 ) in float id;
layout( location = 1 ) in vec2 inPointA;
layout( location = 2 ) in vec2 inPointB;
layout( location = 3 ) in vec2 inPointC;

uniform mat3 u_world_matrix;

uniform mat4 u_projection_matrix;
uniform float u_width;

uniform float u_segments;

void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB, 1.0 ) ).xy;
  vec2 pointC = ( u_world_matrix * vec3( inPointC, 1.0 ) ).xy;

  vec2 xBasis = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 yBasis = vec2( -xBasis.y, xBasis.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  vec2 ABNorm = normalize( vec2( -AB.y, AB.x ) );
  vec2 CBNorm = normalize( vec2( CB.y, -CB.x ) );

  float sigma = sign( dot( AB + CB, yBasis ) );

  if( id == 0.0 )
  {
    vec2 point = 0.5 * yBasis * -sigma * u_width / dot( yBasis, ABNorm );
    gl_Position =  u_projection_matrix * vec4( pointB + point, 0.0, 1.0 );
    return;
  }

  float theta = acos( dot( ABNorm, CBNorm ) );
  theta = sigma *  0.5 * PI - 0.5 * theta + theta  * ( id - 1.0 ) / u_segments;

  vec2 point = 0.5 * u_width * vec2( cos( theta ), sin( theta ) );
  point = pointB + xBasis * point.x + yBasis * point.y;

  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}