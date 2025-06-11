#version 300 es

#define MAX_OBJECT_COUNT 1024

precision highp float;
precision mediump usampler2D;

in vec2 vUv;
out vec4 FragColor;

const uint IDS[ 13 ] = uint[ 13 ](
  2u,
  6u,
  7u,
  8u,
  10u,
  11u,
  12u,
  13u,
  14u,
  16u,
  17u,
  18u,
  22u
);

// G-Buffer textures
uniform sampler2D sourceTexture;
uniform sampler2D positionTexture;
uniform sampler2D objectColorIdTexture;

layout( std140 ) uniform ObjectColorBlock
{
  vec4 objectColors[ MAX_OBJECT_COUNT ];
};

// Projection matrix for converting view-space coordinates to clip-space.
uniform mat4 projection;
// Resolution of the viewport, used for calculating pixel offsets.
uniform vec2 resolution;

uniform float outlineThickness;

float outline_stencil() 
{
  // Sample the color texture in a 3x3 kernel around the current fragment to perform edge detection.
  float pix[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      // uint objectColorId = uint( 
      //   texture(
      //     objectColorIdTexture,
      //     vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      //   ).x 
      // );

      // pix[ y * 5 + x ] = objectColors[ objectColorId ].x;

      vec4 sourceTextureColor =
      texture(
        sourceTexture,
        vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      );

      pix[ y * 5 + x ] = sourceTextureColor.x;
    }
  }

  float slaplacian =
  (
    + pix[ 2 ] * -1.0
    + pix[ 6 ] * -2.0
    + pix[ 7 ] * -4.0
    + pix[ 8 ] * -2.0
    + pix[ 10 ] * -1.0
    + pix[ 11 ] * -4.0
    + pix[ 12 ] * 28.0
    + pix[ 13 ] * -4.0
    + pix[ 14 ] * -1.0
    + pix[ 16 ] * -2.0
    + pix[ 17 ] * -4.0
    + pix[ 18 ] * -2.0
    + pix[ 22 ] * -1.0
  );

  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      // uint objectColorId = uint( 
      //   texture(
      //     objectColorIdTexture,
      //     vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      //   ).x 
      // );

      // pix[ y * 5 + x ] = objectColors[ objectColorId ].x;

      vec4 positionTextureColor =
      texture(
        positionTexture,
        vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      );

      pix[ y * 5 + x ] = length( positionTextureColor.xyzw );
    }
  }

  float plaplacian =
  (
    + pix[ 2 ] * -1.0
    + pix[ 6 ] * -2.0
    + pix[ 7 ] * -4.0
    + pix[ 8 ] * -2.0
    + pix[ 10 ] * -1.0
    + pix[ 11 ] * -4.0
    + pix[ 12 ] * 28.0
    + pix[ 13 ] * -4.0
    + pix[ 14 ] * -1.0
    + pix[ 16 ] * -2.0
    + pix[ 17 ] * -4.0
    + pix[ 18 ] * -2.0
    + pix[ 22 ] * -1.0
  );

  // Clamp the outline value to ensure it's within a valid range [0, 1].
  float outline = clamp( max( slaplacian, plaplacian ), 0.0, 1.0 );

  return outline;
}

vec4 outline_color()
{
  vec4 near_color = vec4( 0.0 );
  float near_depth = 0.0;

  vec4 colors[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      uint objectColorId = uint( 
        texture(
          objectColorIdTexture,
          vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
        ).x 
      );

      colors[ y * 5 + x ] = objectColors[ objectColorId ];
    }
  }

  float depths[ 25 ];
  for( int y = 0; y < 5; y++ )
  {
    for( int x = 0; x < 5; x++ )
    {
      depths[ y * 5 + x ] = 1.0 - texture(
        positionTexture,
        vUv + vec2( float( x - 2 ), float( y - 2 ) ) * outlineThickness / resolution
      ).w;
    }
  }

  for ( int i = 0; i < 13; i++ )
  {
    uint j = IDS[ i ];
    if ( near_depth < depths[ j ] && depths[ j ] < 1.0 )
    {
      near_depth = depths[ j ];
      near_color = colors[ j ];
    }
  }

  return near_color;
}

void main()
{
  float outline = outline_stencil();

  if ( outline > 0.6 )
  {
    FragColor = vec4( 0.0, 1.0, 0.0, 1.0 );
  }
  else
  {
    FragColor = texture( sourceTexture, vUv );
  }
}