#version 300 es
precision highp float;

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;
layout( location = 3 ) in vec3 inPointC;
layout( location = 4 ) in float inUvX;
layout( location = 5 ) in float currentDistance;

uniform mat3 u_world_matrix;
uniform mat3 u_view_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;
uniform float u_total_distance;

out vec2 vUv;

vec2 lineIntersection( vec2 p1, vec2 d1, vec2 p2, vec2 d2 )
{
  float d = d1.y * d2.x - d1.x * d2.y;
  vec2 dp = p2 - p1;

  vec2 r1 = vec2( -d2.y, d2.x );
  float k = dot( r1, dp ) / d;
  return p1 + d1 * k;
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

  // Direction of the bend
  float sigma = sign( dot( AB + CB, normal ) );

  vec2 normToAB = normalize( vec2( -AB.y, AB.x ) );
  vec2 normToCB = normalize( vec2( -CB.y, CB.x ) );

  // Upper corner
  vec2 p1 = pointB + 0.5 * u_width * sigma * normal / dot( -normToCB, normal );
  // Bottom corner
  vec2 p3 = vec2( 0.0 );
  
  
  vec2 leftBottomCornerA = pointA + normToAB * -sigma * u_width * 0.5;
  vec2 rightBottomCornerC = pointC + normToCB * sigma * u_width * 0.5;

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

  vec2 uvPoint = offsetPoint;
  
  // If two segments overlap each other
  if( dot( offsetPoint - intersectionPoint, normal * sigma ) < 0.0 )
  {
    uvPoint = intersectionPoint;
    vec2 normalizedAB = normalize( AB );
    vec2 cAtoInt =  intersectionPoint - leftBottomCornerA;
    float k = dot( cAtoInt, normalizedAB );
    offsetPoint = leftBottomCornerA + k * normalizedAB + normalizedAB * dot( normal * sigma, normalizedAB ) * length( intersectionPoint - offsetPoint );

    if( dot( offsetPoint - pointB, AB ) > 0.0 )
    {
      offsetPoint = leftBottomCornerA + AB;
    }
  }

  p3 = lineIntersection( pointB, normal, offsetPoint, normToAB );

  float uvLeftK = distanceToLine( pointA, normToAB, uvPoint ) / length( pointB - pointA );
  float uvRightK = distanceToLine( pointC, normToCB, uvPoint ) / length( pointB - pointC );

  float uvLeft = mix( inPointA.z, inPointB.z, uvLeftK ); 
  float uvRight = mix( inPointC.z, inPointB.z, uvRightK ); 

  // Left corner
  vec2 p0 = lineIntersection( pointB + normToAB * sigma * u_width * 0.5, AB, p3, normToAB );
  // Right corner
  vec2 p2 = lineIntersection( pointB - normToCB * sigma * u_width * 0.5, CB, p3, normToCB );

  vUv.y = mix( 0.0, 1.0, position.x + position.y + position.z );
  vUv.y = mix( 1.0 - vUv.y, vUv.y, step( 0.0, sigma ) );
  vUv.x = mix( uvLeft, uvRight, inUvX );

  vec2 point = p3 + ( p0 - p3 ) * position.x + ( p1 - p3 ) * position.y + ( p2 - p3 ) * position.z;

  vec3 view_point = u_view_matrix * vec3( point, 1.0 );

  gl_Position = u_projection_matrix * vec4( view_point.xy, 0.0, 1.0 );
}