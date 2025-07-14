#version 300 es
precision highp float;

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 pointA;
layout( location = 2 ) in vec2 pointB;
layout( location = 3 ) in vec2 pointC;

uniform mat4 u_projection_matrix;
uniform float u_width;

void main() 
{
  vec2 tangent = normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) );
  vec2 miter = vec2( -tangent.y, tangent.x );

  vec2 AB = pointB - pointA;
  vec2 CB = pointB - pointC;

  float sigma = sign( dot( AB + CB, miter ) );

  vec2 ABNorm = normalize( vec2( -AB.y, AB.x ) );
  vec2 CBNorm = normalize( vec2( CB.y, -CB.x ) );

  vec2 p0 = 0.5 * u_width * sigma * ( sigma < 0.0 ? ABNorm : CBNorm );
  vec2 p1 = 0.5 * u_width * sigma * ( sigma < 0.0 ? CBNorm : ABNorm );

  vec2 point = pointB + p0 * position.x + p1 * position.y;

  gl_Position =  u_projection_matrix * vec4( point, 0.0, 1.0 );
}