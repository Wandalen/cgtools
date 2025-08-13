#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;
layout( location = 3 ) in vec3 inPointC;

uniform mat3 u_world_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;

out vec2 vUv;

vec2 lineIntersection( vec2 p1, vec2 n1, vec2 p2, vec2 n2 )
{
  vec2 m = ( p2 - p1 ) / n1;
  vec2 n = n2 / n1;
  float d = ( m.x - m.y ) / ( n.y - n.x );
  return d * n2 + p2;
}

float distanceToLine( vec2 a, vec2 n, vec2 p )
{
  vec2 ap = a - p;
  vec2 perp = ap - dot( ap, n ) * n;
  return length( perp );
}

void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA.xy, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB.xy, 1.0 ) ).xy;
  vec2 pointC = ( u_world_matrix * vec3( inPointC.xy, 1.0 ) ).xy;

  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 normal = vec2( -tangent.y, tangent.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  vec2 xBasis = pointB - pointA;
  vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );

  vec2 normToAB = normalize( vec2( -AB.y, AB.x ) );
  vec2 normToCB = normalize( vec2( -CB.y, CB.x ) );

  // Direction of the bend
  float sigma = sign( dot( AB + CB, normal ) );

  vec2 corner1 = pointA + normToAB * sigma * u_width * 0.5;
  vec2 corner2 = pointA - normToAB * sigma * u_width * 0.5;
  vec2 corner3 = pointB - normToAB * sigma * u_width * 0.5;
  vec2 corner4 = pointB + normToAB * sigma * u_width * 0.5;

  vec2 currentPoint = 
  mix
  (
    // x = 0
    mix
    (
      // y < 0
      corner2,
      // y > 0
      corner1,
      step( 0.0, position.y )
    ),
    // x = 1
    mix
    (
      // y < 0
      corner3,
      // y > 0
      corner4,
      step( 0.0, position.y )
    ),
    position.x
  );

  float totalDist = distanceToLine( pointA, normal, pointB );
  float currentDist = distanceToLine( pointA, normal, currentPoint );

  vUv.x = mix( inPointA.z, inPointB.z, currentDist / totalDist );
  vUv.y = mix( 0.0, 1.0, step( 0.0, sign( inPointB.z - inPointA.z ) * position.y ) );
  //vUv.x = mix( inPointA.z, inPointB.z, position.x );

  if( position.x == 0.0 )
  {
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
    return;
  }

  if( sign( position.y ) == -sigma )
  {
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

    vUv.x = inPointB.z;
    gl_Position = u_projection_matrix * vec4( offsetPoint, 0.0, 1.0 );
  }
  else
  {
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
  }
}