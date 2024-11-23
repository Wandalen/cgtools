#version 300 es 
precision mediump float;
#pragma vscode_glsllint_stage : frag

in vec2 vUv;
in vec3 vPos;

uniform sampler2D glyphs; 
uniform float time;

out vec4 frag_color;

float sample_glyph( vec2 uv )
{
  vec3 l = vec3( 1.0 ) - texture( glyphs, uv ).xyz;
  return max( min( l.x, l.y ), min( max( l.x, l.y ), l.z ) );
}

float screenPxRange()
{
  const float pxRange = 4.0;

  vec2 unitRange = vec2( pxRange ) / vec2( textureSize( glyphs, 0 ) );
  vec2 screenTexSize = vec2( 1.0 ) / fwidth( vUv );
  return max( dot( unitRange, screenTexSize ), 1.0 );
}

vec3 pallete( float k )
{
  // vec3 offset = vec3( 0.500, 0.500, 0.500 );
  // vec3 amp = vec3( 0.500, 0.500, 0.500 );
  // vec3 freq = vec3( 1.000, 1.000, 1.000 );
  // vec3 phase = vec3( 0.000, 0.333, 0.667 );

  vec3 offset = vec3( 0.500, 0.500, 0.500 );
  vec3 amp = vec3( 0.500, 0.500, 0.500 );
  vec3 freq = vec3( 0.800, 0.800, 0.500 );
  vec3 phase = vec3( 0.000, 0.200, 0.500 );

  return offset + amp * cos( 2.0 * 3.1415926 * ( freq * k + phase ) );
}

mat3x3 orthBase( vec3 val )
{
  vec3 z = normalize( val );
  vec3 up = z.y < 0.999 ? vec3( 0.0, 1.0, 0.0 ) : vec3( 0.0, 0.0, 1.0 );
  vec3 x = normalize( cross( up, z ) );
  vec3 y = normalize( cross( z, x ) );

  return mat3( x, y, z ); 
}

vec3 cycleNoise( vec3 p, vec3 seed, float persistence, float lacunarity )
{
  vec4 sum = vec4( 0.0 );
  mat3 rot = orthBase( seed );
  for( int i = 0; i < 5; i++ )
  {
    p *= rot;
    p += sin( p.xyz );

    sum += vec4( cross( cos( p ), sin( p.yzx ) ), 1.0 );
    sum /= persistence;
    p *= lacunarity;
  }

  return sum.xyz / sum.w;
}

vec3 gradient( vec3 p, vec3 seed, float persistence, float lacunarity )
{
  const vec2 e = vec2( 0.0001, 0.0 );
  vec3 dx = cycleNoise( p + e.xyy, seed, persistence, lacunarity ) - cycleNoise( p - e.xyy, seed, persistence, lacunarity );
  vec3 dy = cycleNoise( p + e.yxy, seed, persistence, lacunarity ) - cycleNoise( p - e.yxy, seed, persistence, lacunarity );
  vec3 dz = cycleNoise( p + e.yyx, seed, persistence, lacunarity ) - cycleNoise( p - e.yyx, seed, persistence, lacunarity );

  return vec3( dx.x, dy.x, dz.x );
}

mat3x3 rotZ( float angle )
{
  float s = sin( angle );
  float c = cos( angle );

  return mat3
  (
    c,   s,   0.0,
    -s,  c,   0.0,
    0.0, 0.0, 1.0
  );
}


void main()
{

  float glyph = sample_glyph( vUv );

  float screenPxDistance = screenPxRange() * ( glyph - 0.5 );

  float edgeWidth = 2.0;
  float alpha = smoothstep( -edgeWidth, edgeWidth, screenPxDistance );

  if( alpha < 0.001 ) { discard; }

  vec3 noisePos = vec3( gl_FragCoord.xy / 100.0, time );
  vec3 seed = vec3( 1.0, -2.0, 3.0);
  float persistence = 0.5;
  float lacunarity = 2.0;

  vec3 noise = normalize( cycleNoise( noisePos, seed, persistence, lacunarity ) );
  noise = smoothstep( vec3( -1.0 ), vec3( 1.0 ), noise );

  vec3 normal = vec3( 0.0, 0.0, 1.0 );
  vec3 grad = gradient( noisePos, seed, persistence, lacunarity ) * 50.0;
  normal -= grad;


  vec3 lightDir = rotZ( time ) * normalize( vec3( 1.0, 0.0, 1.0 ) );
  vec3 cameraPos = vec3( 0.0, 0.0, 1.0 );
  vec3 viewDir = cameraPos - vPos;

  vec3 H = normalize( lightDir + viewDir );
  float VdotN = clamp( dot( lightDir, normal ), 0.0, 1.0 );
  float HdotN = clamp( dot( H, normal ), 0.0, 1.0 );
  float phongValue = pow( HdotN, 64.0 );

  vec3 color = vec3( pallete( noise.x ) ) * VdotN + vec3( 0.7 ) * phongValue;

  frag_color = vec4( color, alpha * noise.y );
}
