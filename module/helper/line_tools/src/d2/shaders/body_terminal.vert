#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;
layout( location = 3 ) in vec3 inPointC;
layout( location = 4 ) in float currentDistance;

uniform mat3 u_world_matrix;
uniform mat3 u_view_matrix;
uniform mat4 u_projection_matrix;
uniform float u_width;
uniform float u_total_distance;

out vec2 vUv;

// If points are on parallel lines - returns the second point
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

  vec2 normToAB = normalize( vec2( -AB.y, AB.x ) );
  vec2 normToCB = normalize( vec2( -CB.y, CB.x ) );

  // Direction of the bend - left or right
  float curl = sign( cross( vec3( AB, 0.0 ), vec3( CB, 0.0 ) ).z );

  // Direction of the bend - up or down
  float sigma = sign( dot( AB + CB, normal ) );

  // If segments are parallel
  if( sigma == 0.0 ) { sigma = 1.0; }

  //vUv.x = mix( position.x, 1.0 - position.x, float( gl_InstanceID ) );
  vUv.y = step( 0.0, position.y );
  vUv.y = mix( 1.0 - vUv.y, vUv.y, step( 0.0, sigma * curl ) );
  vUv.y = mix( vUv.y, 1.0 - vUv.y, float( gl_InstanceID ) );

  if( position.x == 0.0 )
  {
    vUv.x = inPointA.z;
    vec2 point = pointA + AB * position.x + normToAB * position.y * u_width;
    vec3 view_point = u_view_matrix * vec3( point, 1.0 );
    gl_Position =  u_projection_matrix * vec4( view_point.xy, 0.0, 1.0 );
    return;
  }
  
  vec2 rightUpperCornerA = pointB + normToAB * sigma * u_width * 0.5;
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

  float offsetAmount = dot( normal, normToAB );
  vec2 offsetPoint = pointB + 0.5 * normal * -sigma * u_width / offsetAmount;

  vec2 intersectionPoint = vec2( 0.0 );
  if( abs( normal.x - normToAB.x ) < 1e-6 && abs( normal.y - normToAB.y ) < 1e-6 )
  {
    intersectionPoint = offsetPoint;
  }
  else
  {
    intersectionPoint = lineIntersection( pointB, normal, closestPoint, closestNormal );
  }

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

  vUv.x = mix( inPointA.z, inPointB.z, length( offsetPoint - leftBottomCornerA ) / length( pointB - pointA ) );

  if( sign( position.y ) == -sigma )
  {
    vec3 view_point = u_view_matrix * vec3( offsetPoint, 1.0 );
    gl_Position = u_projection_matrix * vec4( view_point.xy, 0.0, 1.0 );
  }
  else
  {
    vec2 point = offsetPoint + normToAB * sigma * u_width;
    vec3 view_point = u_view_matrix * vec3( point, 1.0 );
    gl_Position =  u_projection_matrix * vec4( view_point.xy, 0.0, 1.0 );
  }
}