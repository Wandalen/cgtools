#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 inPointA;
layout( location = 2 ) in vec2 inPointB;
layout( location = 3 ) in vec2 inPointC;
layout( location = 4 ) in vec2 inPointD;

uniform mat3 u_world_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB, 1.0 ) ).xy;
  vec2 pointC = ( u_world_matrix * vec3( inPointC, 1.0 ) ).xy;
  vec2 pointD = ( u_world_matrix * vec3( inPointD, 1.0 ) ).xy;

  vec2 p0 = pointA;
  vec2 p1 = pointB;
  vec2 p2 = pointC;
  vec2 pos = position;

  if( position.x == 1.0 )
  {
    p0 = pointD;
    p1 = pointC;
    p2 = pointB;
    pos = vec2( 1.0 - position.x, -position.y );
  }

  vec2 tangent = normalize( normalize( p2 - p1 ) + normalize( p1 - p0 ) );
  vec2 normal = vec2( -tangent.y, tangent.x );

  vec2 p01 = p1 - p0;
  vec2 p21 = p1 - p2;

  vec2 p01Norm = normalize( vec2( -p01.y, p01.x ) );
  // Direction of the bend
  float sigma = sign( dot( p01 + p21, normal ) );

  if( sign( pos.y ) == -sigma )
  {
    vec2 point = 0.5 * normal * -sigma * u_width / dot( normal, p01Norm );
    gl_Position = u_projection_matrix * vec4( p1 + point, 0.0, 1.0 );
  }
  else
  {
    vec2 xBasis = p2 - p1;
    vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );
    vec2 point = p1 + xBasis * pos.x + yBasis * pos.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
  }
}