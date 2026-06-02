#version 300 es
precision highp float;

in vec2 vUv;
out vec4 fragColor;

uniform samplerCube envMap;
uniform int face;
uniform float roughness;
uniform float resolution;

const float PI = 3.141592653589793;
const uint SAMPLE_COUNT = 1024u;

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

float radicalInverseVdC( uint bits )
{
  bits = ( bits << 16u ) | ( bits >> 16u );
  bits = ( ( bits & 0x55555555u ) << 1u ) | ( ( bits & 0xAAAAAAAAu ) >> 1u );
  bits = ( ( bits & 0x33333333u ) << 2u ) | ( ( bits & 0xCCCCCCCCu ) >> 2u );
  bits = ( ( bits & 0x0F0F0F0Fu ) << 4u ) | ( ( bits & 0xF0F0F0F0u ) >> 4u );
  bits = ( ( bits & 0x00FF00FFu ) << 8u ) | ( ( bits & 0xFF00FF00u ) >> 8u );
  return float( bits ) * 2.3283064365386963e-10;
}

vec2 hammersley( uint i, uint N )
{
  return vec2( float( i ) / float( N ), radicalInverseVdC( i ) );
}

vec3 importanceSampleGGX( vec2 Xi, vec3 N, float r )
{
  float a = r * r;
  float phi = 2.0 * PI * Xi.x;
  float cosTheta = sqrt( ( 1.0 - Xi.y ) / ( 1.0 + ( a * a - 1.0 ) * Xi.y ) );
  float sinTheta = sqrt( 1.0 - cosTheta * cosTheta );

  vec3 H = vec3( cos( phi ) * sinTheta, sin( phi ) * sinTheta, cosTheta );

  vec3 up = abs( N.z ) < 0.999 ? vec3( 0.0, 0.0, 1.0 ) : vec3( 1.0, 0.0, 0.0 );
  vec3 tangent = normalize( cross( up, N ) );
  vec3 bitangent = cross( N, tangent );

  return tangent * H.x + bitangent * H.y + N * H.z;
}

void main()
{
  vec3 N = faceDirFromUV( vUv, face );
  vec3 R = N;
  vec3 V = R;

  float totalWeight = 0.0;
  vec3 prefilteredColor = vec3( 0.0 );

  for( uint i = 0u; i < SAMPLE_COUNT; i++ )
  {
    vec2 Xi = hammersley( i, SAMPLE_COUNT );
    vec3 H = importanceSampleGGX( Xi, N, roughness );
    vec3 L = normalize( 2.0 * dot( V, H ) * H - V );

    float NdotL = max( dot( N, L ), 0.0 );
    if( NdotL > 0.0 )
    {
      float NdotH = max( dot( N, H ), 0.0 );
      float HdotV = max( dot( H, V ), 0.0 );
      float a2 = roughness * roughness;
      a2 *= a2;
      float D = a2 / ( PI * pow( NdotH * NdotH * ( a2 - 1.0 ) + 1.0, 2.0 ) );
      float pdf = D * NdotH / ( 4.0 * HdotV ) + 0.0001;
      float saTexel = 4.0 * PI / ( 6.0 * resolution * resolution );
      float saSample = 1.0 / ( float( SAMPLE_COUNT ) * pdf + 0.0001 );
      // mipLevel is left unclamped: high-pdf samples make saSample/saTexel < 1, so log2
      // yields a negative LOD. textureLod clamps the level to the cubemap's valid mip range
      // [0, maxLevel] per the GL ES 3.0 spec, so both negative and over-max LODs resolve to
      // the base/last mip — an explicit clamp (as in Three.js) is redundant here.
      float mipLevel = roughness == 0.0 ? 0.0 : 0.5 * log2( saSample / saTexel );

      prefilteredColor += textureLod( envMap, L, mipLevel ).rgb * NdotL;
      totalWeight += NdotL;
    }
  }

  prefilteredColor = totalWeight > 0.0 ? prefilteredColor / totalWeight : vec3( 0.0 );
  fragColor = vec4( prefilteredColor, 1.0 );
}
