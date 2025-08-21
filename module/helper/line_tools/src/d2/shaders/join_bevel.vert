#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;
layout( location = 3 ) in vec3 inPointC;
layout( location = 4 ) in float inUvX;
layout( location = 5 ) in float currentDistance;

uniform mat3 u_world_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;
uniform float u_total_distance;

out vec2 vUv;

vec2 lineIntersection( vec2 p1, vec2 n1, vec2 p2, vec2 n2 )
{
  vec2 m = ( p2 - p1 ) / n1;
  vec2 n = n2 / n1;
  float d = ( m.x - m.y ) / ( n.y - n.x );
  return d * n2 + p2;
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

  // Direction of the bend
  float sigma = sign( dot( AB + CB, normal ) );

  vec2 normToAB = normalize( vec2( -AB.y, AB.x ) );
  vec2 normToCB = normalize( vec2( -CB.y, CB.x ) );

  // Bottom corner
  vec2 p2 = vec2( 0.0 );
  
  vec2 leftBottomCornerA = pointA + normToAB * -sigma * u_width * 0.5;
  vec2 rightBottomCornerC = pointC + normToCB * sigma * u_width * 0.5;
  vec2 rightUpperleftBottomCornerA = pointB + normToAB * sigma * u_width * 0.5;

  vec2 closestPoint;
  vec2 closestNormal;

  // Choose the closest corner
  if( dot( AB, AB ) > dot( CB, CB ) )
  {
    closestPoint = rightBottomCornerC;
    closestNormal = normToCB;
  }
  else
  {
    closestPoint = leftBottomCornerA;
    closestNormal = normToAB;
  }

  vec2 intersectionPoint = lineIntersection( pointB, normal, closestPoint, closestNormal );
  vec2 offsetPoint = pointB + 0.5 * normal * -sigma * u_width / dot( normal, normToAB );

  // If two segments overlap each other
  if( dot( offsetPoint - intersectionPoint, normal * sigma ) < 0.0 )
  {
    vec2 normalizedAB = normalize( AB );
    vec2 cAtoInt =  intersectionPoint - leftBottomCornerA;
    float k = dot( cAtoInt, normalizedAB );
    offsetPoint = leftBottomCornerA + k * normalizedAB + normalizedAB * dot( normal * sigma, normalizedAB ) * length( intersectionPoint - offsetPoint );

    if( dot( offsetPoint - pointB, AB ) > 0.0 )
    {
      offsetPoint = leftBottomCornerA + AB;
    }
  }

  p2 = lineIntersection( pointB, normal, offsetPoint, normToAB );
  

  // Left corner
  vec2 p0 = lineIntersection( pointB + normToAB * sigma * u_width * 0.5, AB, p2, normToAB * sigma );
  // Right corner
  vec2 p1 = lineIntersection( pointB - normToCB * sigma * u_width * 0.5, CB, p2, normToCB * sigma );

  float uvLeft = mix( inPointA.z, inPointB.z, length( p2 - leftBottomCornerA ) / length( pointB - pointA ) ); 
  float uvRight = mix( inPointC.z, inPointB.z, length( p2 - rightBottomCornerC ) / length( pointB - pointC ) ); 

  vUv.y = mix( 0.0, 1.0, position.x + position.y );
  vUv.y = mix( 1.0 - vUv.y, vUv.y, step( 0.0, sigma ) );
  vUv.x = mix( uvLeft, uvRight, inUvX );

  vec2 point = p2 + ( p0 - p2 ) * position.x + ( p1 - p2 ) * position.y;

  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}