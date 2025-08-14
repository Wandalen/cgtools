#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 inPointA;
layout( location = 2 ) in vec2 inPointB;
layout( location = 3 ) in vec2 inPointC;

uniform mat3 u_world_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

vec2 lineIntersection( vec2 p1, vec2 n1, vec2 p2, vec2 n2 )
{
  vec2 m = ( p2 - p1 ) / n1;
  vec2 n = n2 / n1;
  float d = ( m.x - m.y ) / ( n.y - n.x );
  return d * n2 + p2;
}

void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB, 1.0 ) ).xy;
  vec2 pointC = ( u_world_matrix * vec3( inPointC, 1.0 ) ).xy;

  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 normal = vec2( -tangent.y, tangent.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  vec2 xBasis = pointB - pointA;
  vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );

  if( position.x == 0.0 )
  {
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
    return;
  }
  // Direction of the bend
  float sigma = sign( dot( AB + CB, normal ) );

  if( sign( position.y ) == -sigma )
  {
    vec2 normToAB = normalize( vec2( -AB.y, AB.x ) );
    vec2 normToCB = normalize( vec2( -CB.y, CB.x ) );

    vec2 cornerA = pointA + normToAB * -sigma * u_width * 0.5;
    vec2 cornerC = pointC + normToCB * sigma * u_width * 0.5;

    vec2 closestPoint;
    vec2 closestNormal;

    if( dot( AB, AB ) > dot( CB, CB ) )
    {
      closestPoint = cornerC;
      closestNormal = normToCB;
    }
    else
    {
      closestPoint = cornerA;
      closestNormal = normToAB;
    }

    vec2 intersectionPoint = lineIntersection( pointB, normal, closestPoint, closestNormal );
    vec2 offsetPoint = pointB + 0.5 * normal * -sigma * u_width / dot( normal, normToAB );

    if( dot( offsetPoint - intersectionPoint, normal * sigma ) < 0.0 )
    {
      vec2 normalizedAB = normalize( AB );
      vec2 cAtoInt =  intersectionPoint - cornerA;
      float k = dot( cAtoInt, normalizedAB );
      offsetPoint = cornerA + k * normalizedAB + normalizedAB * dot( normal * sigma, normalizedAB ) * length( intersectionPoint - offsetPoint );

      if( dot( offsetPoint - pointB, AB ) > 0.0 )
      {
        offsetPoint = cornerA + AB;
      }
    }

    gl_Position = u_projection_matrix * vec4( offsetPoint, 0.0, 1.0 );
  }
  else
  {
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
  }
}