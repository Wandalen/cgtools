#version 300 es
precision highp float;

in vec2 vUv;

out vec4 FragColor;

uniform sampler2D equirectMap;
uniform mat4 invProjection;
uniform mat4 invView;

const float PI = 3.1415926535897932384626433;

vec3 getWorldDir( vec2 uv )
{
  vec4 clip = vec4( uv * 2.0 - 1.0, -1.0, 1.0 );
  vec4 view = invProjection * clip;
  view /= view.w;
  view.w = 0.0;
  vec3 worldDir = ( invView * view ).xyz;
  return normalize( worldDir );
}

vec2 dirToEquirectUV( vec3 dir )
{
  float phi = atan( dir.z, dir.x );
  float theta = asin( dir.y );
  vec2 uv = vec2( 0.5 + phi / ( 2.0 * PI ), 0.5 - theta / PI );

  if ( uv.x < 0.0005 || uv.x > 0.9995 )
  {
    uv = vec2( 0.0001, uv.y );
  }

  return uv;
}

void main()
{
  vec3 dir = getWorldDir( vUv );
  vec2 uv = dirToEquirectUV( dir );

  if (uv.x > 0.001 && uv.x < 0.999)
  {
    FragColor = texture( equirectMap, uv );
  }
  else
  {
    FragColor = texture( equirectMap, vec2( 0.0001, uv.y ) );
  }
}
