#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;
layout( location = 3 ) in vec3 inPointC;
layout( location = 4 ) in vec3 inPointD;

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


void main() 
{
  vec2 pointA = ( u_world_matrix * vec3( inPointA.xy, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB.xy, 1.0 ) ).xy;
  vec2 pointC = ( u_world_matrix * vec3( inPointC.xy, 1.0 ) ).xy;
  vec2 pointD = ( u_world_matrix * vec3( inPointD.xy, 1.0 ) ).xy;

  vec2 p0 = pointA;
  vec2 p1 = pointB;
  vec2 p2 = pointC;
  vec2 pos = position;

  vUv.y = step( 0.0, pos.y );
  vUv.x = position.x;

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

  vec2 normTo01 = normalize( vec2( -p01.y, p01.x ) );
  vec2 normTo21 = normalize( vec2( -p21.y, p21.x ) );

  // Direction of the bend
  float sigma = sign( dot( p01 + p21, normal ) );

  vec2 leftBottomCorner0 = p0 + normTo01 * -sigma * u_width * 0.5;
  vec2 rightBottomCorner2 = p2 + normTo21 * sigma * u_width * 0.5;

  vec2 closestPoint;
  vec2 closestNormal;

  // Choose the closest corner
  if( dot( p01, p01 ) > dot( p21, p21 ) )
  {
    closestPoint = rightBottomCorner2;
    closestNormal = normTo21;
  }
  else
  {
    closestPoint = leftBottomCorner0;
    closestNormal = normTo01;
  }

  float offsetAmount = dot( normal, normTo01 );
  vec2 intersectionPoint = lineIntersection( p1, normal, closestPoint, closestNormal );
  vec2 offsetPoint = p1 + 0.5 * normal * -sigma * u_width / offsetAmount;

  // If two segments overlap each other
  if( dot( offsetPoint - intersectionPoint, normal * sigma ) < 0.0 )
  {
    vec2 normalized21 = normalize( p21 );
    vec2 c2toInt =  intersectionPoint - rightBottomCorner2;
    float k = dot( c2toInt, normalized21 );
    offsetPoint = rightBottomCorner2 + k * normalized21 + normalized21 * dot( normal * sigma, normalized21 ) * length( intersectionPoint - offsetPoint );

    if( dot( offsetPoint - p1, p21 ) > 0.0 )
    {
      offsetPoint = rightBottomCorner2 + p21;
    }
  }

  if( sign( pos.y ) == -sigma )
  {
    gl_Position = u_projection_matrix * vec4( offsetPoint, 0.0, 1.0 );
  }
  else
  {
    vec2 point = offsetPoint - normTo21 * sigma * u_width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
  }
}