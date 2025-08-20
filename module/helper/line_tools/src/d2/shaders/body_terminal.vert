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

// If points are on parallel lines - returns the second point
vec2 lineIntersection( vec2 p1, vec2 n1, vec2 p2, vec2 n2 )
{
  if( dot( p2 - p1, n2 ) == 0.0 )
  {
    return p2;
  }

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

  vec2 normToAB = normalize( vec2( -AB.y, AB.x ) );
  vec2 normToCB = normalize( vec2( -CB.y, CB.x ) );

  // Direction of the bend - left or right
  float curl = sign( cross( vec3( AB, 0.0 ), vec3( CB, 0.0 ) ).z );

  // Direction of the bend - up or down
  float sigma = sign( dot( AB + CB, normal ) );

  // If segments are parallel
  if( sigma == 0.0 ) { sigma = 1.0; }

  vUv.x = mix( 1.0 - position.x, position.x, step( 0.0, sigma ) );
  vUv.y = step( 0.0, position.y );
  //vUv.y = mix( vUv.y, 1.0 -  vUv.y, step( 0.0, mix( -sigma, sigma, float( gl_InstanceID ) ) ) );
  vUv.y = mix( 1.0 - vUv.y, vUv.y, step( 0.0, sigma * curl ) );
  vUv.y = mix( vUv.y, 1.0 - vUv.y, float( gl_InstanceID ) );

  if( position.x == 0.0 )
  {
    vec2 point = pointA + AB * position.x + normToAB * position.y * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
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
  vec2 intersectionPoint = lineIntersection( pointB, normal, closestPoint, closestNormal );
  vec2 offsetPoint = pointB + 0.5 * normal * -sigma * u_width / offsetAmount;

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

  if( sign( position.y ) == -sigma )
  {
    gl_Position = u_projection_matrix * vec4( offsetPoint, 0.0, 1.0 );
  }
  else
  {
    vec2 point = offsetPoint + normToAB * sigma * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
  }
}