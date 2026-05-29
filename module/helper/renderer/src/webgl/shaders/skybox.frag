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
  return vec2( 0.5 + phi * INV_2PI, theta * INV_PI + 0.5 );
}

void main()
{
  vec2 uv = dirToEquirectUV( vDir );

  // Alpha = 1 marks the skybox as covered geometry so the tone mapping pass maps it
  // ( the environment is tone mapped, unlike the solid clear-color background ).
  frag_color = vec4( texture( equirectMap, uv ).rgb, 1.0 );
  emissive_color = vec4( 0.0 );
}
