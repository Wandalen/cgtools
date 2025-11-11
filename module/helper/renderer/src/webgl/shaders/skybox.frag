#version 300 es
precision highp float;

in vec2 vUv;
out vec4 FragColor;

uniform sampler2D uEquirectMap;
uniform mat4 uInvProjection;
uniform mat4 uInvView;

const float PI = 3.1415926535897932384626433;

vec3 getWorldDir( vec2 uv )
{
  vec4 clip = vec4( uv * 2.0 - 1.0, -1.0, 1.0 );
  vec4 view = uInvProjection * clip;
  view /= view.w;
  view.w = 0.0;
  vec3 worldDir = ( uInvView * view ).xyz;
  return normalize( worldDir );
}

vec2 dirToEquirectUV( vec3 dir )
{
  float phi = atan( -dir.z, dir.x );
  float theta = asin( dir.y );
  return vec2( 0.5 + phi / ( 2.0 * PI ), 0.5 - theta / PI );
}

void main()
{
  vec3 dir = getWorldDir( vUv );
  dir.z = -dir.z;
  vec2 uv = dirToEquirectUV( dir );

  if (uv.x > 0.0005 && uv.x < 0.9995)
  {
    FragColor = texture( uEquirectMap, uv );
  }
  else
  {
    FragColor = texture( uEquirectMap, vec2( 0.0001, uv.y ) );
  }
}
