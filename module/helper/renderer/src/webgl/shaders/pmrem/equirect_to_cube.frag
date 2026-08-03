#version 300 es
precision highp float;

in vec2 vUv;
out vec4 fragColor;

uniform sampler2D equirectMap;
uniform int face;

const float PI = 3.141592653589793;
const float INV_PI = 1.0 / PI;
const float INV_2PI = 1.0 / ( 2.0 * PI );

vec3 faceDirFromUV( vec2 uv, int f )
{
  vec2 ndc = uv * 2.0 - 1.0;
  vec3 dir;
  if( f == 0 ) dir = vec3(  1.0, -ndc.y, -ndc.x );
  else if( f == 1 ) dir = vec3( -1.0, -ndc.y,  ndc.x );
  else if( f == 2 ) dir = vec3(  ndc.x,  1.0,  ndc.y );
  else if( f == 3 ) dir = vec3(  ndc.x, -1.0, -ndc.y );
  else if( f == 4 ) dir = vec3(  ndc.x, -ndc.y,  1.0 );
  else              dir = vec3( -ndc.x, -ndc.y, -1.0 );
  return normalize( dir );
}

vec2 dirToEquirectUV( vec3 dir )
{
  vec3 d = normalize( dir );
  float phi = atan( d.z, d.x );
  float theta = asin( clamp( d.y, -1.0, 1.0 ) );
  return vec2( 0.5 + phi * INV_2PI, theta * INV_PI + 0.5 );
}

void main()
{
  vec3 dir = faceDirFromUV( vUv, face );
  vec2 uv = dirToEquirectUV( dir );
  uv.y = 1.0 - uv.y;
  fragColor = texture( equirectMap, uv );
}
