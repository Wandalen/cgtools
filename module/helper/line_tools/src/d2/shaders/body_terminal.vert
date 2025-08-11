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

  if( position.x == 0.0 )
  {
    vec2 xBasis = pointB - pointA;
    vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
    return;
  }


  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 normal = vec2( -tangent.y, tangent.x );

  vec2 pAB = pointB - pointA;
  vec2 pCB = pointB - pointC;

  vec2 ABNorm = normalize( vec2( -pAB.y, pAB.x ) );
  // Direction of the bend
  float sigma = sign( dot( pAB + pCB, normal ) );

  if( sign( position.y ) == -sigma )
  {
    vec2 dirA = dot( normalize( pAB ), normal * sigma ) * pAB;
    vec2 dirC = dot( normalize( pCB ), normal * sigma ) * pCB;

    float maxDist = min( length( dirA ), length( dirC ) );
    vec2 correctionPoint = normal * -sigma * maxDist;

    vec2 point = 0.5 * normal * -sigma * u_width / dot( normal, ABNorm );

    if( length( point ) > maxDist )
    {
      point = correctionPoint - ( point - correctionPoint );
    }
    
    gl_Position = u_projection_matrix * vec4( pointB + point, 0.0, 1.0 );
  }
  else
  {
    vec2 xBasis = pointB - pointA;
    vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
  }
}