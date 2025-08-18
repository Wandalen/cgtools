#version 300 es
precision highp float;

layout( location = 0 ) in vec4 position;
layout( location = 1 ) in vec3 inPointA;
layout( location = 2 ) in vec3 inPointB;
layout( location = 3 ) in vec3 inPointC;
layout( location = 4 ) in float inUvIndex;

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


  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 normal = vec2( -tangent.y, tangent.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  float sigma = sign( dot( AB + CB, normal ) );

  vec2 ABNorm = normalize( vec2( -AB.y, AB.x ) );
  vec2 CBNorm = normalize( vec2( CB.y, -CB.x ) );

  vec2 p1 = 0.5 * u_width * sigma * normal / dot( CBNorm, normal );
  vec2 p3 = vec2( 0.0 );
  

  //if( position.w == 1.0 )
  {
    vec2 normToAB = normalize( vec2( -AB.y, AB.x ) );
    vec2 normToCB = normalize( vec2( -CB.y, CB.x ) );

    vec2 cornerA = pointA + normToAB * -sigma * u_width * 0.5;
    vec2 cornerC = pointC + normToCB * sigma * u_width * 0.5;
    vec2 cornerA2 = pointB + normToAB * sigma * u_width * 0.5;

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

    p3 = lineIntersection( pointB, normal, offsetPoint, ABNorm ) - pointB;
    //p3 = lineIntersection( pointB, normal, pointB - normToAB * sigma * u_width * 0.5, ABNorm ) - pointB;
  }

  // vec2 p0 = p3 + ABNorm * sigma * u_width;
  // vec2 p2 = p3 + CBNorm * sigma * u_width;

  vec2 p0 = lineIntersection( pointB + ABNorm * sigma * u_width * 0.5, AB, p3 + pointB, ABNorm * sigma ) - pointB;
  vec2 p2 = lineIntersection( pointB + CBNorm * sigma * u_width * 0.5, CB, p3 + pointB, CBNorm * sigma ) - pointB;

  vUv.y = mix( 0.0, 1.0, 1.0 - position.w );
  //vUv.x = inPointB.z * position.x + ( inPointB.z + 1.0 ) * position.z + ( inPointB.z + 0.5 ) * position.y + ( inPointB.z + 0.5 ) * position.w;

  if( abs( inUvIndex ) < 1e-5 ) { vUv.x = 0.0; }
  else if ( abs( inUvIndex - 1.0 ) < 1e-5 ) { vUv.x = 0.5; }
  else if ( abs( inUvIndex - 2.0 ) < 1e-5 ) { vUv.x = 1.0; }

  //vUv.x = vUv.y;


  //if( inUvIndex.y == 1.0 ) { vUv.y = 1.0 - vUv.y; }

  vec2 point =
  pointB + 
  mix( p3, p0, position.x ) * ( 1.0 - step( position.x, 0.0 ) )  + 
  mix( p3, p1, position.y ) * ( 1.0 - step( position.y, 0.0 ) ) + 
  mix( p3, p2, position.z ) * ( 1.0 - step( position.z, 0.0 ) ) + 
  p3 * position.w;

  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}