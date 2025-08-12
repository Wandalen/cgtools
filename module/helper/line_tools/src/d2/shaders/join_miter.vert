#version 300 es
precision highp float;

layout( location = 0 ) in vec4 position;
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


  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 normal = vec2( -tangent.y, tangent.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  float sigma = sign( dot( AB + CB, normal ) );

  vec2 ABNorm = normalize( vec2( -AB.y, AB.x ) );
  vec2 CBNorm = normalize( vec2( CB.y, -CB.x ) );

  vec2 p0 = 0.5 * u_width * sigma * ( sigma < 0.0 ? ABNorm : CBNorm );
  vec2 p1 = 0.5 * u_width * sigma * normal / dot( CBNorm, normal );
  vec2 p2 = 0.5 * u_width * sigma * ( sigma < 0.0 ? CBNorm : ABNorm );
  vec2 p3 = 0.5 * normal * -sigma * u_width / dot( normal, ABNorm );

  {
    vec2 xBasis = pointB - pointA;
    vec2 yBasis = normalize( vec2( -xBasis.y, xBasis.x ) );

    vec2 cornerA = pointA + ABNorm * -sigma * u_width * 0.5;
    vec2 cornerC = pointC + CBNorm * sigma * u_width * 0.5;

    vec2 cAB = pointB - cornerA;
    vec2 cCB = pointB - cornerC;

    vec2 dirA = dot( normalize( cAB ), normal * sigma ) * cAB;
    vec2 dirC = dot( normalize( cCB ), normal * sigma ) * cCB;

    float maxDist = min( length( dirA ), length( dirC ) );
    vec2 correctionPoint = normal * -sigma * maxDist;
    vec2 offsetPoint = 0.5 * normal * -sigma * u_width / dot( normal, ABNorm );

    float offsetLength = length( offsetPoint );
    if( offsetLength > maxDist )
    {
      // float hypot = 2.0 * length( offsetPoint - correctionPoint );
      offsetPoint = correctionPoint - offsetPoint + correctionPoint;
      // offsetPoint += yBasis * sigma * hypot * dot( normal, -yBasis );
      p3 = offsetPoint;
    }
  }

  vec2 point = pointB + p0 * position.x + p1 * position.y + p2 * position.z +p3 * position.w;

  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}