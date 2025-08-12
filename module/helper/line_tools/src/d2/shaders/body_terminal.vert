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
  float width = u_width;
  vec2 pointA = ( u_world_matrix * vec3( inPointA, 1.0 ) ).xy;
  vec2 pointB = ( u_world_matrix * vec3( inPointB, 1.0 ) ).xy;
  vec2 pointC = ( u_world_matrix * vec3( inPointC, 1.0 ) ).xy;

  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 normal = vec2( -tangent.y, tangent.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  vec2 xBasis = pointB - pointA;
  vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );

  vec2 ABNorm = normalize( vec2( -AB.y, AB.x ) );
  vec2 CBNorm = normalize( vec2( -CB.y, CB.x ) );
  // Direction of the bend
  float sigma = sign( dot( AB + CB, normal ) );

  vec2 cornerA = pointA + ABNorm * -sigma * width * 0.5;
  vec2 cornerC = pointC + CBNorm * sigma * width * 0.5;

  vec2 cAB = pointB - cornerA;
  vec2 cCB = pointB - cornerC;

  vec2 closestPoint;
  vec2 closestNormal;

  if( dot( cAB, cAB ) > dot( cCB, cCB ) )
  {
    closestPoint = cornerC;
    closestNormal = CBNorm * sigma;
  }
  else
  {
    closestPoint = cornerA;
    closestNormal = -ABNorm * sigma;
  }

  vec2 intersectionPoint = lineIntersection( pointB, normal, closestPoint, closestNormal );
  vec2 offsetPoint = pointB + 0.5 * normal * -sigma * width / dot( normal, ABNorm );


  //offsetPoint = intersectionPoint;
  if( dot( offsetPoint - intersectionPoint, normal * sigma ) < 0.0 )
  {

    //offsetPoint
    vec2 normalizedAB = normalize( AB );
    float k = dot( normalize( intersectionPoint - cornerA ), normalizedAB ) * length( intersectionPoint - cornerA );
    offsetPoint = cornerA + k * normalizedAB + normalizedAB * dot( normal * sigma, normalizedAB ) * length( intersectionPoint - offsetPoint );
  }

  // vec2 dirA = dot( normalize( cAB ), normal * sigma ) * cAB;
  // vec2 dirC = dot( normalize( cCB ), normal * sigma ) * cCB;

  // float maxDist = min( length( dirA ), length( dirC ) );
  // vec2 correctionPoint = normal * -sigma * maxDist;
  // vec2 offsetPoint = 0.5 * normal * -sigma * width / dot( normal, ABNorm );

  // float offsetLength = length( offsetPoint );
  // if( offsetLength > maxDist )
  // {
  //   float hypot = 2.0 * length( offsetPoint - correctionPoint );
  //   offsetPoint = correctionPoint - offsetPoint + correctionPoint;
  //   offsetPoint += yBasis * sigma * hypot * dot( normal, -yBasis );
  //   //width = width * maxDist / offsetLength;
  // }

  if( position.x == 0.0 )
  {
    // vec2 xBasis = pointB - pointA;
    // vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
    return;
  }

  if( sign( position.y ) == -sigma )
  {
    

    // vec2 point = 0.5 * normal * -sigma * u_width / dot( normal, ABNorm );

    // if( length( point ) > maxDist )
    // {
    //   point = correctionPoint - ( point - correctionPoint );
    // }
    
    gl_Position = u_projection_matrix * vec4( offsetPoint, 0.0, 1.0 );
  }
  else
  {
    // vec2 xBasis = pointB - pointA;
    // vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );
    vec2 point = pointA + xBasis * position.x + yBasis * position.y * width;
    gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
  }
}