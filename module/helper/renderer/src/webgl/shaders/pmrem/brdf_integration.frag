#version 300 es
precision highp float;
// Fix(android-silhouette-dots): `precision highp float` says nothing about integers, and
// the fragment-stage default is `mediump int`, which Mali runs as 16-bit. There
// radicalInverseVdC's `bits << 16u | bits >> 16u` is 0 for every sample, so all 1024
// Hammersley points collapse onto H = N. The integrated LUT then goes to ~0 at grazing
// N.V with any real roughness, and polished metal renders black along its silhouette.
// Desktop and Apple GPUs run mediump int at 32 bits, so the fault never shows there.
// Pitfall: any shader doing 32-bit integer bit-twiddling must declare highp int itself.
precision highp int;

in vec2 vUv;
out vec4 fragColor;

const float PI = 3.141592653589793;
const uint SAMPLE_COUNT = 1024u;

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

vec3 importanceSampleGGX( vec2 Xi, vec3 N, float roughness )
{
  float a = roughness * roughness;
  float phi = 2.0 * PI * Xi.x;
  float cosTheta = sqrt( ( 1.0 - Xi.y ) / ( 1.0 + ( a * a - 1.0 ) * Xi.y ) );
  float sinTheta = sqrt( 1.0 - cosTheta * cosTheta );

  vec3 H = vec3( cos( phi ) * sinTheta, sin( phi ) * sinTheta, cosTheta );

  vec3 up = abs( N.z ) < 0.999 ? vec3( 0.0, 0.0, 1.0 ) : vec3( 1.0, 0.0, 0.0 );
  vec3 tangent = normalize( cross( up, N ) );
  vec3 bitangent = cross( N, tangent );

  return tangent * H.x + bitangent * H.y + N * H.z;
}

float geometrySchlickGGX( float NdotV, float roughness )
{
  float a = roughness;
  float k = ( a * a ) / 2.0;
  return NdotV / ( NdotV * ( 1.0 - k ) + k );
}

float geometrySmith( vec3 N, vec3 V, vec3 L, float roughness )
{
  float NdotV = max( dot( N, V ), 0.0 );
  float NdotL = max( dot( N, L ), 0.0 );
  return geometrySchlickGGX( NdotV, roughness ) * geometrySchlickGGX( NdotL, roughness );
}

void main()
{
  float NdotV = vUv.x;
  float roughness = vUv.y;

  vec3 V;
  V.x = sqrt( 1.0 - NdotV * NdotV );
  V.y = 0.0;
  V.z = NdotV;

  float A = 0.0;
  float B = 0.0;

  vec3 N = vec3( 0.0, 0.0, 1.0 );

  for( uint i = 0u; i < SAMPLE_COUNT; i++ )
  {
    vec2 Xi = hammersley( i, SAMPLE_COUNT );
    vec3 H = importanceSampleGGX( Xi, N, roughness );
    vec3 L = normalize( 2.0 * dot( V, H ) * H - V );

    float NdotL = max( L.z, 0.0 );
    float NdotH = max( H.z, 0.0 );
    // Clamped above too: V and H are unit vectors, but dot() can round to 1.0000001 when
    // H ~ V (N.V near 1, low roughness). pow( 1.0 - VdotH, 5.0 ) below would then take a
    // negative base, which is undefined and NaN on Mali — and one NaN sample poisons the
    // whole accumulated texel. Only reachable once the Hammersley sequence works.
    float VdotH = clamp( dot( V, H ), 0.0, 1.0 );

    if( NdotL > 0.0 )
    {
      float G = geometrySmith( N, V, L, roughness );
      float G_Vis = ( G * VdotH ) / ( max( NdotH, 0.001 ) * max( NdotV, 0.001 ) );
      float Fc = pow( 1.0 - VdotH, 5.0 );

      A += ( 1.0 - Fc ) * G_Vis;
      B += Fc * G_Vis;
    }
  }

  A /= float( SAMPLE_COUNT );
  B /= float( SAMPLE_COUNT );

  fragColor = vec4( A, B, 0.0, 1.0 );
}
