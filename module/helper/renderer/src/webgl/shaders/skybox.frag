#version 300 es
precision highp float;

in vec3 vDir;

layout( location = 0 ) out vec4 frag_color;
layout( location = 1 ) out vec4 emissive_color;

uniform sampler2D equirectMap;

const float PI = 3.141592653589793;
const float INV_PI = 1.0 / PI;
const float INV_2PI = 1.0 / ( 2.0 * PI );

vec2 dirToEquirectUV( vec3 dir )
{
  vec3 d = normalize( dir );
  float phi = atan( d.z, d.x );
  float theta = asin( d.y );
  return vec2( 0.5 + phi * INV_2PI, 0.5 - theta * INV_PI );
}

void main()
{
  vec2 uv = dirToEquirectUV( vDir );

  if ( uv.x < 0.001 || uv.x > 0.999 )
  {
    uv.x = 0.0001;
  }

  frag_color = texture( equirectMap, uv );
  emissive_color = vec4( 0.0 );
}
